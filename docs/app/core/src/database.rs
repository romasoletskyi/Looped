use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter, Result};
use std::str::FromStr;

use serde_derive::{Deserialize, Serialize};

use crate::data::{GeneralPerson, Phrase, WordCloud};

pub const SERVER: &str = "server";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DatabaseDifference {
    texts: HashMap<usize, usize>,
    responses: HashMap<usize, usize>,
}

impl DatabaseDifference {
    fn new() -> Self {
        DatabaseDifference {
            texts: HashMap::new(),
            responses: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DifferenceManager {
    differences: HashMap<String, DatabaseDifference>,
}

impl DifferenceManager {
    fn new() -> Self {
        DifferenceManager {
            differences: HashMap::new(),
        }
    }

    fn insert_text(&mut self, index: usize, start: usize) {
        for (_, difference) in &mut self.differences {
            difference.texts.entry(index).or_insert(start);
        }
    }

    fn insert_response(&mut self, index: usize, start: usize) {
        for (_, difference) in &mut self.differences {
            difference.responses.entry(index).or_insert(start);
        }
    }

    fn difference(&self, base: &Database, client: &str) -> Database {
        let mut database = Database::new();
        database.updated(SERVER);

        if let Some(difference) = self.differences.get(client) {
            for (&index, &text) in &difference.texts {
                base.add_difference(
                    &mut database,
                    index,
                    Some(text),
                    difference.responses.get(&index).copied(),
                );
            }

            for (&index, &response) in &difference.responses {
                if difference.texts.get(&index).is_none() {
                    base.add_difference(&mut database, index, None, Some(response));
                }
            }
        }

        database
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
    pub(crate) phrases: Vec<Phrase>,
    phrase_indices: HashMap<WordCloud, usize>,
    manager: DifferenceManager,
}

impl Database {
    pub fn new() -> Self {
        Database {
            phrases: Vec::new(),
            phrase_indices: HashMap::new(),
            manager: DifferenceManager::new(),
        }
    }

    pub fn from_str(s: &str) -> Option<Database> {
        serde_json::from_str(s).ok()
    }

    pub fn from_slice(slice: &[u8]) -> Option<Database> {
        serde_json::from_slice(slice).ok()
    }

    pub fn updated(&mut self, client: &str) {
        self.manager
            .differences
            .insert(client.to_string(), DatabaseDifference::new());
    }

    pub fn difference(&mut self, client: &str) -> Database {
        self.manager.difference(&self, client)
    }

    pub fn total_clone(&self) -> Database {
        let mut database = Database::new();
        database.updated(SERVER);

        for index in 0..self.phrases.len() {
            self.add_difference(&mut database, index, Some(0), Some(0));
        }

        database
    }

    // merges starting from database.difference indices
    // all response indices have to be present in database.phrase_indices
    pub fn merge(&mut self, database: Database) {
        let mut index_to_cloud: HashMap<usize, WordCloud> = HashMap::new();
        for (cloud, index) in database.phrase_indices {
            index_to_cloud.insert(index, cloud);
        }

        let mut merged_indices = HashMap::new();
        let difference = database.manager.differences.get(SERVER).unwrap();

        for (&index, &start) in &difference.texts {
            let texts_slice = &database.phrases[index].texts[start..];

            if let Some(merged_index) = self.insert_texts_at(
                database.phrases[index].texts[0].as_str(),
                texts_slice.iter().cloned(),
            ) {
                merged_indices.insert(index, merged_index);
            }
        }

        for (&index, &start) in &difference.responses {
            let responses: Vec<(usize, GeneralPerson)> = database.phrases[index].responses[start..]
                .iter()
                .map(|response| {
                    let merged_index = self
                        .phrase_indices
                        .get(index_to_cloud.get(&response.0).unwrap())
                        .unwrap();
                    (*merged_index, response.1)
                })
                .collect();
            self.insert_responses_to(*merged_indices.get(&index).unwrap(), responses);
        }
    }
}

impl Database {
    pub(crate) fn get_start_index(&self) -> usize {
        *self
            .phrase_indices
            .get(&WordCloud::from_str("").unwrap())
            .unwrap()
    }

    pub(crate) fn insert_texts_at<I: IntoIterator<Item = String>>(
        &mut self,
        base_text: &str,
        texts: I,
    ) -> Option<usize> {
        if let Ok(cloud) = WordCloud::from_str(base_text) {
            let phrase_index = self.phrase_indices.get(&cloud).copied();
            let real_index = phrase_index.unwrap_or(self.phrases.len());

            if let Some(index) = phrase_index {
                let text_vec: Vec<String> = texts.into_iter().collect();
                if !text_vec.is_empty() {
                    self.manager
                        .insert_text(index, self.phrases[index].texts.len());
                    self.phrases[index].texts.extend(text_vec);
                }
            } else {
                self.manager.insert_text(real_index, 0);
                self.phrases.push(Phrase::new());
                self.phrases[real_index].texts.extend(texts);
                self.phrase_indices.insert(cloud, real_index);
            }

            Some(real_index)
        } else {
            None
        }
    }

    pub(crate) fn insert_responses_to<I: IntoIterator<Item = (usize, GeneralPerson)>>(
        &mut self,
        index: usize,
        responses: I,
    ) {
        self.manager
            .insert_response(index, self.phrases[index].responses.len());
        self.phrases[index].responses.extend(responses);
    }

    fn add_difference(
        &self,
        database: &mut Database,
        index: usize,
        text: Option<usize>,
        response: Option<usize>,
    ) {
        let length = database.phrases.len();
        let difference = database.manager.differences.get_mut(SERVER).unwrap();

        let texts = if let Some(text_start) = text {
            difference.texts.insert(length, 0);
            self.phrases[index].texts[text_start..].to_vec()
        } else {
            difference.texts.insert(length, 1);
            vec![self.phrases[index].texts[0].clone()]
        };

        let responses = if let Some(response_start) = response {
            difference.responses.insert(length, 0);
            for &(response_index, _) in &self.phrases[index].responses[response_start..] {
                database.phrase_indices.insert(
                    WordCloud::from_str(&self.phrases[response_index].texts[0]).unwrap(),
                    response_index,
                );
            }
            self.phrases[index].responses[response_start..].to_vec()
        } else {
            Vec::new()
        };

        database.phrases.push(Phrase { texts, responses });
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Database {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self).map_err(|_| Error::default())?
        )
    }
}

fn vec_to_multiset<T: std::hash::Hash + std::cmp::Eq + Clone>(vec: &Vec<T>) -> HashMap<T, u32> {
    let mut map = HashMap::new();

    for element in vec {
        *map.entry(element.clone()).or_default() += 1;
    }

    map
}

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        let mut to_other = HashMap::new();

        if self.phrase_indices.len() != other.phrase_indices.len() {
            return false;
        }

        for (cloud, index) in &self.phrase_indices {
            if let Some(&other_index) = other.phrase_indices.get(cloud) {
                to_other.insert(index, other_index);
            } else {
                return false;
            }
        }

        if self.phrases.len() != other.phrases.len() {
            return false;
        }

        for (index, phrase) in (&self.phrases).iter().enumerate() {
            let other_phrase = &other.phrases[to_other[&index]];
            if vec_to_multiset(&phrase.texts) != vec_to_multiset(&other_phrase.texts) {
                return false;
            }

            let mapped_responses: Vec<(usize, GeneralPerson)> = phrase
                .responses
                .iter()
                .map(|(index, person)| (to_other[index], *person))
                .collect();
            if vec_to_multiset(&mapped_responses) != vec_to_multiset(&other_phrase.responses) {
                return false;
            }
        }

        return true;
    }
}

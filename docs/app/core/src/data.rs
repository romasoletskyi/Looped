use std::collections::{HashMap, BTreeSet};
use std::fmt::{self, Error, Formatter};
use std::hash::Hash;
use std::iter::zip;
use std::ops::Add;
use std::str::{self, FromStr};
use std::vec;

use rand::Rng;
use rand::{rngs::ThreadRng, thread_rng};
use serde_derive::{Deserialize, Serialize};

use crate::console_log;

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
enum Job {
    Farmer,
    Merchant,
    Priest,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
struct Character {
    hostile: i8,
    rebellious: i8,
}

impl Character {
    fn to_vec(self) -> Vec<f32> {
        vec![self.hostile as f32, self.rebellious as f32]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
struct Person {
    job: Job,
    character: Character,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
struct GeneralPerson(Option<Person>);

impl GeneralPerson {
    fn distance(&self, other: &GeneralPerson) -> f32 {
        match (&self.0, &other.0) {
            (None, None) => 0.0,
            (Some(lhs), Some(rhs)) => {
                (lhs.job != rhs.job) as i32 as f32
                    + GeneralPerson::cosine_distance(&lhs.character, &rhs.character)
            }
            _ => 2.0,
        }
    }

    fn scalar_product<T: Add<Output = T> + Default>(lhs: &[T], rhs: &[T]) -> T
    where
        for<'a> &'a T: std::ops::Mul<&'a T, Output = T>,
        T: std::default::Default,
    {
        zip(lhs.iter(), rhs.iter())
            .map(|(x, y)| x * y)
            .fold(Default::default(), |x, y| x + y)
    }

    fn cosine_distance(lhs: &Character, rhs: &Character) -> f32 {
        let lslice = lhs.to_vec();
        let rslice = rhs.to_vec();
        1.0 - GeneralPerson::scalar_product(&lslice, &rslice)
            / f32::sqrt(
                GeneralPerson::scalar_product(&lslice, &lslice)
                    * GeneralPerson::scalar_product(&rslice, &rslice),
            )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Phrase {
    texts: Vec<String>,
    responses: Vec<(usize, GeneralPerson)>,
}

impl Phrase {
    fn new() -> Self {
        Phrase {
            texts: Vec::new(),
            responses: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq)]
pub struct WordCloud(String);

impl WordCloud {
    pub fn new(s: &str) -> Self {
        WordCloud(s.to_string())
    }
}

impl FromStr for WordCloud {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(WordCloud::new(
            s.replace(&['(', ')', ',', '\"', '.', ';', ':', '\'', '?', '!', '-'], "")
                .to_lowercase()
                .split(' ')
                .fold(String::new(), |a, b| a + b + " ")
                .trim_end()
        ))
    }
}

impl PartialEq for WordCloud {
    fn eq(&self, other: &Self) -> bool {
        self.0.split(' ').collect::<BTreeSet<&str>>()
            == other.0.split(' ').collect::<BTreeSet<&str>>()
    }
}

impl Hash for WordCloud {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for word in self.0.split(' ').collect::<BTreeSet<&str>>() {
            word.hash(state);
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseDifference {
    texts: HashMap<usize, usize>,
    responses: HashMap<usize, usize>,
}

impl DatabaseDifference {
    fn new() -> Self {
        DatabaseDifference {
            texts: HashMap::new(),
            responses: HashMap::new()
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Database {
    phrases: Vec<Phrase>,
    phrase_indices: HashMap<WordCloud, usize>,
    difference: DatabaseDifference,
}

impl Database {
    pub fn new() -> Self {
        Database {
            phrases: Vec::new(),
            phrase_indices: HashMap::new(),
            difference: DatabaseDifference::new(),
        }
    }

    pub fn from_str(s: &str) -> Option<Database> {
        serde_json::from_str(s).ok()
    }

    pub fn from_slice(slice: &[u8]) -> Option<Database> {
        serde_json::from_slice(slice).ok()
    }

    pub fn difference(&mut self) -> Database {
        console_log!("{:?}", self.difference);
        let mut database = Database::new();        

        for (&index, &text) in &self.difference.texts {
            self.add_difference(&mut database, index, Some(text), self.difference.responses.get(&index).copied());
        }

        for (&index, &response) in &self.difference.responses {
            if self.difference.texts.get(&index).is_none() {
                self.add_difference(&mut database, index, None, Some(response));
            }
        }

        self.difference = DatabaseDifference::new();
        database
    }

    // merges starting from database.difference indices
    // all response indices have to be present in database.phrase_indices
    pub fn merge(&mut self, database: Database) {
        let mut index_to_merged = HashMap::new();
        for (index, start) in database.difference.texts {
            let texts_slice = &database.phrases[index].texts[start..];
            if let Some(merged_index) =
                self.insert_texts_at(texts_slice[0].as_str(), texts_slice.iter().cloned())
            {
                index_to_merged.insert(index, merged_index);
            }
        }

        let mut index_to_cloud : HashMap<usize, WordCloud>= HashMap::new();
        for (cloud, index) in database.phrase_indices {
            index_to_cloud.insert(index, cloud);
        }

        for (index, start) in database.difference.responses {
            let responses: Vec<(usize, GeneralPerson)> = database.phrases[index].responses[start..]
            .iter()
            .map(|response| {
                let merged_index = self.phrase_indices.get(index_to_cloud.get(&response.0).unwrap()).unwrap();
                (*merged_index, response.1) 
            }).collect();
            self.insert_responses_to(index_to_merged[&index], responses);
        }
    }
}

impl Database {
    fn get_start_index(&self) -> usize {
        *self.phrase_indices.get(&WordCloud("".to_string())).unwrap()
    }

    fn insert_texts_at<I: IntoIterator<Item = String>>(
        &mut self,
        base_text: &str,
        texts: I,
    ) -> Option<usize> {
        if let Ok(cloud) = WordCloud::from_str(base_text) {
            let phrase_index = self.phrase_indices.get(&cloud).copied();
            let real_index = phrase_index.unwrap_or(self.phrases.len());

            if let Some(index) = phrase_index {
                self.difference
                    .texts
                    .entry(index)
                    .or_insert(self.phrases[index].texts.len());
                self.phrases[index].texts.extend(texts)
            } else {
                self.difference.texts.insert(real_index, 0);
                self.phrases.push(Phrase::new());
                self.phrases[real_index].texts.extend(texts);
                self.phrase_indices.insert(cloud, real_index);
            }

            Some(real_index)
        } else {
            None
        }
    }

    fn insert_responses_to<I: IntoIterator<Item = (usize, GeneralPerson)>>(
        &mut self,
        index: usize,
        responses: I,
    ) {
        self.difference
            .responses
            .entry(index)
            .or_insert(self.phrases[index].responses.len());
        self.phrases[index].responses.extend(responses);
    }

    fn add_difference(&self, database: &mut Database, index: usize, text: Option<usize>, response: Option<usize>) {
        let texts = if let Some(text_start) = text {
            self.phrases[index].texts[text_start..].to_vec()
        } else {
            Vec::new()
        };
        let responses = if let Some(response_start) = response {
            for &(response_index, _) in &self.phrases[index].responses[response_start..] {
                database.phrase_indices.insert(WordCloud::from_str(&self.phrases[response_index].texts[0]).unwrap(), response_index);
            }
            self.phrases[index].responses[response_start..].to_vec()
        } else {
            Vec::new()
        };

        let length = database.phrases.len();
        database.difference.texts.insert(length, 0);
        database.difference.responses.insert(length, 0);
        database.phrases.push(Phrase { texts, responses });
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self).map_err(|_| Error::default())?
        )
    }
}

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        let mut to_other = HashMap::new();

        for (cloud, index) in &self.phrase_indices {
            if let Some(&other_index) = other.phrase_indices.get(cloud) {
                to_other.insert(index, other_index);
            } else {
                return false;
            }
        }

        for (index, phrase) in (&self.phrases).iter().enumerate() {
            let other_phrase = &other.phrases[to_other[&index]];
            if phrase.texts != other_phrase.texts {
                return false;
            }

            if phrase.responses.len() != other_phrase.responses.len() {
                return false;
            }
            
            for i in 0..phrase.responses.len() {
                if to_other[&phrase.responses[i].0] != other_phrase.responses[i].0 {
                    return false;
                }
                if phrase.responses[i].1 != other_phrase.responses[i].1 {
                    return false;
                }
            }
        }

        return true;
    }
}

const CHAT_VARIANTS: usize = 4;

pub struct Chat {
    database: *mut Database,
    gen: ThreadRng,
    query_options: Vec<usize>,
    query: Option<usize>,
    person: GeneralPerson,
    you_talk: bool,
}

impl Chat {
    pub fn new(database: &mut Database, you_talk: bool, person_descrirption: &str) -> Self {
        database.insert_texts_at("", vec!["".to_string()]);

        Chat {
            database,
            gen: thread_rng(),
            query_options: Vec::new(),
            query: None,
            person: if let Ok(person) = serde_json::from_str(person_descrirption) {
                GeneralPerson(Some(person))
            } else {
                GeneralPerson(None)
            },
            you_talk,
        }
    }

    pub fn get_phrases(&mut self) -> Vec<String> {
        let index = self.query.unwrap_or(self.get_database().get_start_index());
        let options = self.get_database().phrases[index].responses.clone();

        let probability: Vec<f32> = options
            .iter()
            .map(|person| f32::exp(-person.1.distance(&self.person)))
            .collect();
        let queries = self.sample_queries(options, probability);

        let text_options = queries.iter().map(|query| self.choose_random_phrase(*query)).collect();
        self.query_options = queries;

        text_options
    }

    pub fn add_phrase(&mut self, text: &str) {
        if let Some(phrase_index) = self
            .get_database()
            .insert_texts_at(text, vec![text.to_string()])
        {   
            self.add_response(phrase_index);
            self.finish_turn(phrase_index);
        }
    }

    pub fn choose_phrase(&mut self, option_number: usize) {
        let response_index = self.query_options[option_number];
        self.add_response(response_index);
        self.finish_turn(response_index);
    }
}

impl Chat {
    fn get_database(&mut self) -> &mut Database {
        unsafe { &mut (*self.database) }
    }

    fn add_response(&mut self, response_index: usize) {
        let previous_index = self.query.unwrap_or(self.get_database().get_start_index());
        let person = self.person;
        self.get_database()
            .insert_responses_to(previous_index, vec![(response_index, person)]);
    }

    fn finish_turn(&mut self, response_index: usize) {
        self.query = Some(response_index);
        self.you_talk = !self.you_talk;
    }

    fn sample_queries(
        &mut self,
        options: Vec<(usize, GeneralPerson)>,
        probability: Vec<f32>,
    ) -> Vec<usize> {
        let mut cumulative: Vec<f32> = probability
            .iter()
            .scan(0.0, |sum, &x| {
                *sum += x;
                Some(*sum)
            })
            .collect();

        if let Some(&sum) = cumulative.last() {
            for x in &mut cumulative {
                *x /= sum;
            }
            let mut queries = Vec::new();

            while queries.len() < std::cmp::min(CHAT_VARIANTS, options.len()) {
                let p = self.gen.gen_range(0.0..1.0f32);
                let index = cumulative
                    .binary_search_by(|x| f32::total_cmp(x, &p))
                    .map_or_else(|x| x, |x| x);

                let query = options[index].0;
                if !queries.iter().any(|&x| x == query) {
                    queries.push(query);
                }
            }

            queries
        } else {
            Vec::new()
        }
    }

    fn choose_random_phrase(&mut self, query_index: usize) -> String {
        let text_number = self.get_database().phrases[query_index].texts.len();
        let index = self.gen.gen_range(0..text_number);
        self.get_database().phrases[query_index].texts[index].clone()
    }
}

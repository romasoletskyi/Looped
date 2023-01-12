use std::collections::{HashMap, HashSet};
use std::fmt::{self, Error, Formatter};
use std::hash::Hash;
use std::iter::zip;
use std::ops::Add;
use std::str::{self, FromStr};
use std::vec;

use rand::Rng;
use rand::{rngs::ThreadRng, thread_rng};
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::console_log;

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Job {
    Farmer,
    Merchant,
    Priest,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Character {
    hostile: i8,
    rebellious: i8,
}

impl Character {
    fn to_vec(self) -> Vec<f32> {
        vec![self.hostile as f32, self.rebellious as f32]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Person {
    job: Job,
    character: Character,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct GeneralPerson(Option<Person>);

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Phrase {
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
struct WordCloud(String);

impl FromStr for WordCloud {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(WordCloud(
            s.replace(&['(', ')', ',', '\"', '.', ';', ':', '\''][..], "")
                .split(' ')
                .fold(String::new(), |a, b| a + b + " "),
        ))
    }
}

impl PartialEq for WordCloud {
    fn eq(&self, other: &Self) -> bool {
        self.0.split(' ').collect::<HashSet<&str>>()
            == other.0.split(' ').collect::<HashSet<&str>>()
    }
}

impl Hash for WordCloud {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for word in self.0.split(' ').collect::<HashSet<&str>>() {
            word.hash(state);
        }
    }
}

fn iter_to_jsarray<I>(iterator: I) -> Box<[JsValue]>
where
    I: Iterator,
    wasm_bindgen::JsValue: From<<I as Iterator>::Item>,
{
    let mut vec = Vec::new();
    for value in iterator {
        vec.push(JsValue::from(value));
    }
    vec.into_boxed_slice()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DatabaseDifference {
    texts: HashMap<usize, usize>,
    responses: HashMap<usize, usize>,
    start_indices: Option<usize>,
}

impl DatabaseDifference {
    fn new() -> Self {
        DatabaseDifference {
            texts: HashMap::new(),
            responses: HashMap::new(),
            start_indices: None,
        }
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct Database {
    phrases: Vec<Phrase>,
    phrase_indices: HashMap<WordCloud, usize>,
    start_indices: Vec<(usize, GeneralPerson)>,
    difference: DatabaseDifference,
}

#[wasm_bindgen]
impl Database {
    pub fn new() -> Self {
        Database {
            phrases: Vec::new(),
            phrase_indices: HashMap::new(),
            start_indices: Vec::new(),
            difference: DatabaseDifference::new(),
        }
    }

    pub fn from_str(s: &str) -> Option<Database> {
        serde_json::from_str(s).ok()
    }

    pub fn difference(&mut self) -> Database {
        let mut database = Database::new();

        for (&index, &start) in &self.difference.texts {
            let texts = self.phrases[index].texts[start..].to_vec();
            let responses = if let Some(&response_start) = self.difference.responses.get(&index) {
                self.phrases[index].responses[response_start..].to_vec()
            } else {
                Vec::new()
            };

            let length = database.phrases.len();
            database.difference.texts.insert(length, 0);
            database.difference.responses.insert(length, 0);
            database.phrases.push(Phrase { texts, responses })
        }

        database.difference.start_indices = Some(0);
        database.start_indices = self.start_indices[self
            .difference
            .start_indices
            .unwrap_or(self.start_indices.len())..]
            .to_vec();

        self.difference = DatabaseDifference::new();
        database
    }

    // merges starting from database.difference indices
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

        for (index, start) in database.difference.responses {
            self.insert_responses_to(
                Some(index_to_merged[&index]),
                database.phrases[index].responses[start..]
                    .iter()
                    .map(|response| {
                        (
                            *index_to_merged.get(&response.0).unwrap_or(&response.0),
                            response.1,
                        )
                    }),
            );
        }

        self.insert_responses_to(
            None,
            database.start_indices[database
                .difference
                .start_indices
                .unwrap_or(database.start_indices.len())..]
                .iter()
                .copied(),
        )
    }
}

impl Database {
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        serde_json::from_slice(slice).ok()
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
        query: Option<usize>,
        responses: I,
    ) {
        if let Some(index) = query {
            self.difference
                .responses
                .entry(index)
                .or_insert(self.phrases[index].responses.len());
            &mut self.phrases[index].responses
        } else {
            if self.difference.start_indices.is_none() {
                self.difference.start_indices = Some(self.start_indices.len());
            }
            &mut self.start_indices
        }
        .extend(responses);
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

const CHAT_VARIANTS: usize = 4;

#[wasm_bindgen]
pub struct Chat {
    database: *mut Database,
    gen: ThreadRng,
    query_options: Vec<usize>,
    query: Option<usize>,
    person: GeneralPerson,
    you_talk: bool,
}

#[wasm_bindgen]
impl Chat {
    pub fn new(database: &mut Database, you_talk: bool, person_descrirption: &str) -> Self {
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

    pub fn get_phrases(&mut self) -> Box<[JsValue]> {
        let options = if let Some(index) = self.query {
            self.get_database().phrases[index].responses.clone()
        } else {
            self.get_database().start_indices.clone()
        };

        let probability: Vec<f32> = options
            .iter()
            .map(|person| f32::exp(-person.1.distance(&self.person)))
            .collect();
        let queries = self.sample_queries(options, probability);

        let text_options = iter_to_jsarray(
            queries
                .iter()
                .map(|query| self.choose_random_phrase(*query)),
        );
        self.query_options = queries;

        text_options
    }

    pub fn add_phrase(&mut self, text: &str) {
        if let Some(real_index) = self
            .get_database()
            .insert_texts_at(text, vec![text.to_string()])
        {
            let query = self.query;
            let person = self.person;
            self.get_database()
                .insert_responses_to(query, vec![(real_index, person)]);
            self.query = Some(real_index);
        }

        console_log!("{:?}", self.get_database());
    }

    pub fn choose_phrase(&mut self, option_number: usize) {
        self.query = Some(self.query_options[option_number]);
        self.you_talk = !self.you_talk;
    }
}

impl Chat {
    fn get_database(&mut self) -> &mut Database {
        unsafe { &mut (*self.database) }
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

            while options.len() < std::cmp::min(CHAT_VARIANTS, options.len()) {
                let p = self.gen.gen_range(0.0..1.0f32);
                let query = cumulative
                    .binary_search_by(|x| f32::total_cmp(x, &p))
                    .map_or_else(|x| x, |x| x);

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

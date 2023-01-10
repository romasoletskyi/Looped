use std::collections::{HashMap, HashSet};
use std::fmt::{self, Formatter, Error};
use std::str::{self, FromStr};

// use serde::{Deserialize, de::Error as serdeDesearilizeError};
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::console_log;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Serialize, Deserialize, Debug)]
pub enum Job {
    Farmer,
    Merchant,
    Priest,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    hostile: i8,
    rebellious: i8,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct Person {
    job: Job,
    character: Character,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct Phrase {
    text: String,
    responses: Vec<(usize, Option<Person>)>,
}

impl Phrase {
    fn new(text: &str) -> Self {
        Phrase { text: text.to_string(), responses: Vec::new()}
    }
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq)]
struct WordCloud(String);

impl FromStr for WordCloud {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(WordCloud(s.replace(&['(', ')', ',', '\"', '.', ';', ':', '\''][..], "").split(' ').fold(String::new(), |a, b| a + b + " ")))
    }
}

impl PartialEq for WordCloud {
    fn eq(&self, other: &Self) -> bool {
        self.0.split(' ').collect::<HashSet<&str>>() == other.0.split(' ').collect::<HashSet<&str>>()
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct Database {
    phrases: Vec<Phrase>,
    text_indices: HashMap<WordCloud, usize>,
    start_indices: Vec<usize>
}

fn iter_to_jsarray<I>(iterator: I) -> Box<[JsValue]>
where I: Iterator,
      wasm_bindgen::JsValue: From<<I as Iterator>::Item>
{   
    let mut vec = Vec::new();
    for value in iterator {
        vec.push(JsValue::from(value));
    }
    vec.into_boxed_slice()
}

#[wasm_bindgen]
impl Database {
    pub fn new() -> Self {
        Database { phrases: Vec::new() , text_indices: HashMap::new(), start_indices: Vec::new() }
    }
}

impl Database {
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        serde_json::from_slice(slice).ok()
    }

    pub fn merge(&mut self, database: Database) {
        unimplemented!()
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).map_err(|_| Error::default())?)
    }
}

impl FromStr for Database {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[wasm_bindgen]
pub struct Chat {
    database: *mut Database,
    query: Option<usize>,
    query_options: Vec<usize>,
    you_talk: bool
}

#[wasm_bindgen]
impl Chat {
    pub fn new(database: &mut Database, you_talk: bool) -> Self {
        Chat { database, query: None, query_options: Vec::new(), you_talk}
    }

    pub fn get_phrases(&mut self) -> Box<[JsValue]> {
        let database = self.get_database();

        if let Some(index) = self.query {
            self.query_options = database.phrases[index].responses.iter().map(|response| response.0).collect();
        } else {
            self.query_options = database.start_indices.iter().copied().collect();
        }

        iter_to_jsarray(self.query_options.iter().map(|i| &self.get_database().phrases[*i].text))
    }

    pub fn add_phrase(&mut self, text: &str) {
        let database = self.get_database();
        
        let phrase_index = database.phrases.len();
        if let Some(index) = self.query {
            database.phrases[index].responses.push((phrase_index, None));
        } else {
            database.start_indices.push(phrase_index);
        }
        
        let index = database.phrases.len();
        database.phrases.push(Phrase::new(text));
        self.query = Some(index);

        console_log!("{:?}", self.get_database());
    }

    pub fn choose_phrase(&mut self, option_number: usize) {
        self.query = Some(self.query_options[option_number]);
        self.you_talk = !self.you_talk;
    }
}

impl Chat {
    #[allow(clippy::mut_from_ref)]
    fn get_database(&self) -> &mut Database {
        unsafe {&mut (*self.database)}
    }
}
use std::fmt::{self, Formatter, Error};
use std::iter;
use std::str;

use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Serialize, Deserialize)]
pub enum Job {
    Farmer,
    Merchant,
    Priest,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Character {
    animosity: i8,
    rebelliousness: i8,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Person {
    job: Job,
    character: Character,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Phrase {
    text: String,
    responses: Vec<usize>,
    speaker: Option<Person>,
}

impl Phrase {
    fn new(text: &str) -> Self {
        Phrase { text: text.to_string(), responses: Vec::new(), speaker: None}
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Database {
    phrases: Vec<Phrase>,
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
        Database { phrases: Vec::new() , start_indices: Vec::new() }
    }
}

impl Database {
    fn from_string(string: &str) -> Option<Self> {
        serde_json::from_str(string).ok()
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).map_err(|error| Error::default())?)
    }
}

#[wasm_bindgen]
struct Chat {
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

        self.query_options = if let Some(index) = self.query {
            database.phrases[index].responses.iter()
        } else {
            database.start_indices.iter()
        }.map(|i| *i).collect();

        iter_to_jsarray(self.query_options.iter().map(|i| &self.get_database().phrases[*i].text))
    }

    pub fn add_phrase(&self, text: &str) {
        let database = self.get_database();
        
        let phrase_index = database.phrases.len();
        if let Some(index) = self.query {
            database.phrases[index].responses.push(phrase_index);
        } else {
            database.start_indices.push(phrase_index);
        }

        database.phrases.push(Phrase::new(text));
    }

    pub fn choose_phrase(&mut self, option_number: usize) {
        self.query = Some(self.query_options[option_number]);
        self.you_talk = !self.you_talk;
    }
}

impl Chat {
    fn get_database(&self) -> &mut Database {
        unsafe {&mut (*self.database)}
    }
}
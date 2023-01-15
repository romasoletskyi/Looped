use wasm_bindgen::prelude::*;

use crate::data::{Chat, Database};

// light-weight wrapper around crate::data for direct wasm use

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

#[wasm_bindgen]
pub struct ClientDatabase(Database);

#[wasm_bindgen]
impl ClientDatabase {
    pub fn new() -> Self {
        ClientDatabase(Database::new())
    }

    pub fn from_str(s: &str) -> Option<ClientDatabase> {
        Database::from_str(s).map(|database| ClientDatabase(database))
    }

    pub fn merge(&mut self, database: ClientDatabase) {
        self.0.merge(database.0)
    }

    pub fn difference(&mut self) -> ClientDatabase {
        ClientDatabase(self.0.difference())
    }
}

#[wasm_bindgen]
pub struct ClientChat(Chat);

#[wasm_bindgen]
impl ClientChat {
    pub fn new(database: &mut ClientDatabase, you_talk: bool, person_description: &str) -> Self {
        ClientChat(Chat::new(&mut database.0, you_talk, person_description))
    }

    pub fn get_phrases(&mut self) -> Box<[JsValue]> {
        iter_to_jsarray(self.0.get_phrases().iter())
    }

    pub fn add_phrase(&mut self, text: &str) {
        self.0.add_phrase(text);
    }

    pub fn choose_phrase(&mut self, option_number: usize) {
        self.0.choose_phrase(option_number);
    }
}
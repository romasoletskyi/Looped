use std::collections::HashMap;
use std::iter::zip;
use rand::Rng;
use rand::{rngs::ThreadRng, thread_rng};

use crate::data::GeneralPerson;
use crate::database::Database;

const CHAT_VARIANTS: usize = 4;

pub struct Chat {
    database: *mut Database,
    gen: ThreadRng,
    query_options: Vec<usize>,
    query: Option<usize>,
    person: GeneralPerson
}

impl Chat {
    pub fn new(database: &mut Database, you_talk: bool, person_descrirption: &str) -> Self {
        database.insert_texts_at("", vec!["".to_string()]);

        Chat {
            database,
            gen: thread_rng(),
            query_options: Vec::new(),
            query: None,
            person: GeneralPerson::new(serde_json::from_str(person_descrirption).unwrap(), you_talk)
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
        self.person.youtalk = !self.person.youtalk;
    }

    fn sample(&mut self, options: &mut Vec<usize>, proba: &mut Vec<f32>) -> Option<usize> {
        let mut cumulative: Vec<f32> = proba
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

            let p = self.gen.gen_range(0.0..1.0f32);
            let index = cumulative
                .binary_search_by(|x| f32::total_cmp(x, &p))
                .map_or_else(|x| x, |x| x);

            let option = options.swap_remove(index);
            proba.swap_remove(index);

            Some(option)
        } else {
            None
        }
    }

    fn sample_queries(
        &mut self,
        options: Vec<(usize, GeneralPerson)>,
        probability: Vec<f32>,
    ) -> Vec<usize> {
        let mut options_map = HashMap::new();
        for (option, proba) in zip(options, probability) {
            *options_map.entry(option.0).or_insert(0.0) += proba;
        }

        let mut unique_option = Vec::new();
        let mut unique_proba = Vec::new();

        for (option, proba) in options_map {
            unique_option.push(option);
            unique_proba.push(proba);
        }

        let mut queries = Vec::new();
        while let Some(option) = self.sample(&mut unique_option, &mut unique_proba) {
            queries.push(option);
            if queries.len() == CHAT_VARIANTS {
                break;
            }
        }

        queries
    }

    fn choose_random_phrase(&mut self, query_index: usize) -> String {
        let text_number = self.get_database().phrases[query_index].texts.len();
        let index = self.gen.gen_range(0..text_number);
        self.get_database().phrases[query_index].texts[index].clone()
    }
}

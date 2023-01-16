use crate::{data::{WordCloud, Chat, Database}, console_log};
use std::{str::FromStr};
use rand::{Rng, rngs::ThreadRng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[test]
fn test_wordcloud() {
    assert_eq!(WordCloud::from_str("").unwrap(), WordCloud::new(""));
    assert_eq!(WordCloud::from_str("Hello, how are you?").unwrap(), WordCloud::new("hello how are you"));
    assert_eq!(WordCloud::from_str("fine, thanks!").unwrap(), WordCloud::new("fine thanks"));
}

fn initialize_chat(database: &mut Database, rng: &mut ChaCha8Rng) -> Chat {
    let person = format!("{{job: {}, character: {{hostile: {}, rebellious: {}}}}}", 
    ["Farmer", "Merchant", "Priest", ][rng.gen_range(0..3)],
    rng.gen_range(-5..=5),
    rng.gen_range(-5..=5));

    Chat::new(database, rng.gen_bool(0.5), &person)
}

fn generate_words(rng: &mut ChaCha8Rng) -> Vec<String> {
    let mut words = Vec::new();

    for _ in 0..20 {
        let word_length = rng.gen_range(2..=6);
        let mut word = String::new();
        for _ in 0..word_length {
            word.push(('a' as u8 + rng.gen_range(0..26) as u8) as char);
        }
        words.push(word);
    }

    words
}

fn generate_text(words: &Vec<String>, rng: &mut ChaCha8Rng) -> String {
    let mut text = String::new();
    let text_length = rng.gen_range(1..=4);

    for _ in 0..text_length {
        text += &words[rng.gen_range(0..words.len())];
        text += " ";
    }

    text
}

#[test]
fn test_database_merge_basic() {
    let mut server = Database::new();
    let mut client = Database::new();

    let mut rng = ChaCha8Rng::seed_from_u64(71);
    let words = generate_words(&mut rng);

    for _ in 0..3 {
        let mut chat = initialize_chat(&mut client, &mut rng);
        let chat_length = 2; // rng.gen_range(5..20);

        for _ in 0..chat_length {
            let phrases = chat.get_phrases();            
            if rng.gen_bool(1.0 / (1.0 + phrases.len() as f64)) {
                chat.add_phrase(&generate_text(&words, &mut rng))
            } else {
                chat.choose_phrase(rng.gen_range(0..phrases.len()))
            }
        }

        server.merge(client.difference());
        assert_eq!(client, server);   
    }  
}
use std::str::FromStr;
use std::collections::HashSet;
use std::iter::zip;
use rand::{Rng, rngs::ThreadRng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::data::WordCloud;
use crate::database::{Database, SERVER};
use crate::chat::Chat;

#[test]
fn test_wordcloud() {
    assert_eq!(WordCloud::from_str("").unwrap(), WordCloud::new(""));
    assert_eq!(WordCloud::from_str("Hello, how are you?").unwrap(), WordCloud::new("hello how are you"));
    assert_eq!(WordCloud::from_str("fine, thanks!").unwrap(), WordCloud::new("fine thanks"));
}

fn initialize_chat(database: &mut Database, rng: &mut ChaCha8Rng) -> Chat {
    let person = format!(
        r#"{{"job": "{}", "character": {{"hostile": {}, "rebellious": {}}}}}"#, 
    ["Farmer", "Merchant", "Priest"][rng.gen_range(0..3)],
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

fn client_chat(client: &mut Database, rng: &mut ChaCha8Rng, words: &Vec<String>) -> Database {
    let mut chat = initialize_chat(client, rng);
    let chat_length = rng.gen_range(5..20);

    for _ in 0..chat_length {
        let phrases = chat.get_phrases();            
        if rng.gen_bool(1.0 / (1.0 + phrases.len() as f64)) {
            chat.add_phrase(&generate_text(&words, rng))
        } else {
            chat.choose_phrase(rng.gen_range(0..phrases.len()))
        }
    }

    client.difference(SERVER)
}

#[test]
fn test_database_merge_basic() {
    let mut server = Database::new();
    let mut client = Database::new();
    client.updated(SERVER);

    let mut rng = ChaCha8Rng::seed_from_u64(71);
    let words = generate_words(&mut rng);

    for _ in 0..10 {
        server.merge(client_chat(&mut client, &mut rng, &words));
        client.updated(SERVER);
        assert_eq!(client, server);   
    }  
}

#[test]
fn test_database_merge_concurrent() {
    let mut server = Database::new();

    let client_number = 5;
    let mut clients = vec![Database::new(); client_number];
    let ips: Vec<String> = (0..client_number).map(|x| x.to_string()).collect();
    let mut registered_ips = HashSet::new();

    let mut rng = ChaCha8Rng::seed_from_u64(71);
    let words = generate_words(&mut rng);

    for _ in 0..100 {
        let i = rng.gen_range(0..client_number);
        let ip = i.to_string();
        let client = &mut clients[i];

        if registered_ips.contains(&ip) {
            if rng.gen_range(0.0..1.0) < 0.25 {
                *client = Database::new();
                client.updated(SERVER);

                client.merge(server.total_clone());
                client.updated(SERVER);
                server.updated(&ip);
                assert_eq!(client, &mut server);
            }
        } else {
            registered_ips.insert(ip.clone());

            *client = Database::new();
            client.updated(SERVER);

            client.merge(server.total_clone());
            client.updated(SERVER);
            server.updated(&ip);
            assert_eq!(client, &mut server);
        }
        
        let difference = server.difference(&ip);
        server.merge(client_chat(client, &mut rng, &words));

        client.merge(difference);
        client.updated(SERVER);
        server.updated(&ip);

        assert_eq!(client, &mut server);
    }

    // final update
    for (ip, mut client) in zip(ips, clients) {
        client.merge(server.difference(&ip));
        client.updated(SERVER);
        server.updated(&ip);
        assert_eq!(client, server);
    }
}
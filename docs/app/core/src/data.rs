use std::collections::BTreeSet;
use std::hash::Hash;
use std::iter::zip;
use std::ops::Add;
use std::str::{self, FromStr};
use std::vec;

use serde_derive::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Job {
    Farmer,
    Merchant,
    Priest,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Character {
    hostile: i8,
    rebellious: i8,
}

impl Character {
    fn to_vec(self) -> Vec<f32> {
        vec![self.hostile as f32, self.rebellious as f32]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub (crate) struct Person {
    job: Job,
    character: Character,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub (crate) struct GeneralPerson {
    person: Person,
    pub (crate) youtalk: bool
}

impl GeneralPerson {
    pub (crate) fn new(person: Person, youtalk: bool) -> Self {
        GeneralPerson{ person, youtalk }
    }

    pub (crate) fn distance(&self, other: &GeneralPerson) -> f32 {
        if self.youtalk != other.youtalk {
            return 2.0;
        }
        (self.person.job != other.person.job) as i32 as f32 + GeneralPerson::cosine_distance(&self.person.character, &self.person.character)
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub (crate) struct Phrase {
    pub (crate) texts: Vec<String>,
    pub (crate) responses: Vec<(usize, GeneralPerson)>,
}

impl Phrase {
    pub (crate) fn new() -> Self {
        Phrase {
            texts: Vec::new(),
            responses: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, Clone)]
pub (crate) struct WordCloud(String);

impl WordCloud {
    pub (crate) fn new(s: &str) -> Self {
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

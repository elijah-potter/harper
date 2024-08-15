use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd)]
pub struct WordMetadata {
    pub kind: Option<WordKind>,
    pub tense: Option<Tense>,
    pub possessive: Option<bool>
}

impl WordMetadata {
    pub fn or(&self, other: &Self) -> Self {
        Self {
            kind: self.kind.or(other.kind),
            tense: self.tense.or(other.tense),
            possessive: self.possessive.or(other.possessive)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd)]
pub enum Tense {
    Past,
    Present,
    Future
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd)]
#[serde(tag = "kind")]
pub enum WordKind {
    Verb,
    Noun {
        is_proper: Option<bool>,
        is_plural: Option<bool>
    },
    Adjective,
    Adverb
}

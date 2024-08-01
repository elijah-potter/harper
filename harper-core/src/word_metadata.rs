use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordMetadata {
    kind: Option<WordKind>,
    tense: Option<Tense>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tense {
    Past,
    Present,
    Future,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WordKind {
    Verb,
    Noun { is_proper: bool },
    Adjective,
    Adverb,
}

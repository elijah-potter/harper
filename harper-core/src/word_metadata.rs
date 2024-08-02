use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd)]
pub struct WordMetadata {
    kind: Option<WordKind>,
    tense: Option<Tense>,
    plural: Option<bool>
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd)]
pub enum Tense {
    Past,
    Present,
    Future
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd)]
pub enum WordKind {
    Verb,
    Noun { is_proper: bool },
    Adjective,
    Adverb
}

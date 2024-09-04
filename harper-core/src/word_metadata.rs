use is_macro::Is;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Hash)]
pub struct WordMetadata {
    pub noun: Option<NounData>,
    pub verb: Option<VerbData>,
    pub adjective: Option<AdjectiveData>,
    pub adverb: Option<AdverbData>,
    pub conjunction: Option<ConjunctionData>,
    pub swear: Option<bool>
}

impl WordMetadata {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        macro_rules! merge {
            ($a:expr, $b:expr) => {
                match ($a, $b) {
                    (Some(a), Some(b)) => Some(a.or(&b)),
                    (Some(a), None) => Some(a),
                    (None, Some(b)) => Some(b),
                    (None, None) => None
                }
            };
        }

        Self {
            noun: merge!(self.noun, other.noun),
            verb: merge!(self.verb, other.verb),
            adjective: merge!(self.adjective, other.adjective),
            adverb: merge!(self.adverb, other.adverb),
            conjunction: merge!(self.conjunction, other.conjunction),
            swear: self.swear.or(other.swear)
        }
    }

    pub fn is_noun(&self) -> bool {
        self.noun.is_some()
    }

    pub fn is_conjunction(&self) -> bool {
        self.conjunction.is_some()
    }

    pub fn is_verb(&self) -> bool {
        self.verb.is_some()
    }

    pub fn is_adjective(&self) -> bool {
        self.adjective.is_some()
    }

    pub fn is_adverb(&self) -> bool {
        self.adverb.is_some()
    }

    pub fn is_possessive_noun(&self) -> bool {
        matches!(
            self.noun,
            Some(NounData {
                is_possessive: Some(true),
                ..
            })
        )
    }

    pub fn is_plural_noun(&self) -> bool {
        matches!(
            self.noun,
            Some(NounData {
                is_plural: Some(true),
                ..
            })
        )
    }

    pub fn is_proper_noun(&self) -> bool {
        matches!(
            self.noun,
            Some(NounData {
                is_proper: Some(true),
                ..
            })
        )
    }

    pub fn is_pronoun(&self) -> bool {
        matches!(
            self.noun,
            Some(NounData {
                is_pronoun: Some(true),
                ..
            })
        )
    }

    pub fn is_linking_verb(&self) -> bool {
        matches!(
            self.verb,
            Some(VerbData {
                is_linking: Some(true),
                ..
            })
        )
    }

    /// Checks whether a word is _definitely_ a swear.
    pub fn is_swear(&self) -> bool {
        matches!(self.swear, Some(true))
    }

    /// Same thing as [`Self::or`], except in-place rather than a copy.
    pub fn append(&mut self, other: &Self) -> &mut Self {
        *self = self.or(other);
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Is, Hash)]
pub enum Tense {
    Past,
    Present,
    Future
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub struct VerbData {
    pub is_linking: Option<bool>,
    pub tense: Option<Tense>
}

impl VerbData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            is_linking: self.is_linking.or(other.is_linking),
            tense: self.tense.or(other.tense)
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub struct NounData {
    pub is_proper: Option<bool>,
    pub is_plural: Option<bool>,
    pub is_possessive: Option<bool>,
    pub is_pronoun: Option<bool>
}

impl NounData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            is_proper: self.is_proper.or(other.is_proper),
            is_plural: self.is_plural.or(other.is_plural),
            is_possessive: self.is_possessive.or(other.is_possessive),
            is_pronoun: self.is_pronoun.or(other.is_pronoun)
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub struct AdjectiveData {}

impl AdjectiveData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, _other: &Self) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub struct AdverbData {}

impl AdverbData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, _other: &Self) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash)]
pub struct ConjunctionData {}

impl ConjunctionData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, _other: &Self) -> Self {
        Self {}
    }
}

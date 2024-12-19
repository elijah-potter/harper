use is_macro::Is;
use paste::paste;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Hash)]
pub struct WordMetadata {
    pub noun: Option<NounData>,
    pub verb: Option<VerbData>,
    pub adjective: Option<AdjectiveData>,
    pub adverb: Option<AdverbData>,
    pub conjunction: Option<ConjunctionData>,
    pub swear: Option<bool>,
    /// Whether the word is considered especially common.
    #[serde(default = "default_common")]
    pub common: bool,
}

/// Needed for `serde`
fn default_common() -> bool {
    false
}

macro_rules! generate_metadata_queries {
    ($($category:ident has $($sub:ident),*).*) => {
        paste! {
            pub fn is_likely_homograph(&self) -> bool {
                if [$($(self.[< is_ $sub _ $category >](),)*)*].iter().map(|b| *b as u8).sum::<u8>() > 1 {
                    return true;
                }

                [$(
                    self.[< is_ $category >](),
                )*].iter().map(|b| *b as u8).sum::<u8>() > 1
            }

            $(
                #[doc = concat!("Checks if the word is definitely a ", stringify!($category), ".")]
                pub fn [< is_ $category >](&self) -> bool {
                    self.$category.is_some()
                }

                $(
                    #[doc = concat!("Checks if the word is definitely a ", stringify!($category), " and more specifically is labeled as (a) ", stringify!($sub), ".")]
                    pub fn [< is_ $sub _ $category >](&self) -> bool {
                        matches!(
                            self.$category,
                            Some([< $category:camel Data >]{
                                [< is_ $sub >]: Some(true),
                                ..
                            })
                        )
                    }


                    #[doc = concat!("Checks if the word is definitely a ", stringify!($category), " and more specifically is labeled as __not__ (a) ", stringify!($sub), ".")]
                    pub fn [< is_not_ $sub _ $category >](&self) -> bool {
                        matches!(
                            self.$category,
                            Some([< $category:camel Data >]{
                                [< is_ $sub >]: Some(false),
                                ..
                            })
                        )
                    }
                )*
            )*
        }
    };
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
                    (None, None) => None,
                }
            };
        }

        Self {
            noun: merge!(self.noun, other.noun),
            verb: merge!(self.verb, other.verb),
            adjective: merge!(self.adjective, other.adjective),
            adverb: merge!(self.adverb, other.adverb),
            conjunction: merge!(self.conjunction, other.conjunction),
            swear: self.swear.or(other.swear),
            common: self.common || other.common,
        }
    }

    generate_metadata_queries!(
        noun has proper, plural, possessive, pronoun.
        verb has linking.
        conjunction has.
        adjective has.
        adverb has
    );

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
    Future,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct VerbData {
    pub is_linking: Option<bool>,
    pub tense: Option<Tense>,
}

impl VerbData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            is_linking: self.is_linking.or(other.is_linking),
            tense: self.tense.or(other.tense),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct NounData {
    pub is_proper: Option<bool>,
    pub is_plural: Option<bool>,
    pub is_possessive: Option<bool>,
    pub is_pronoun: Option<bool>,
}

impl NounData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        Self {
            is_proper: self.is_proper.or(other.is_proper),
            is_plural: self.is_plural.or(other.is_plural),
            is_possessive: self.is_possessive.or(other.is_possessive),
            is_pronoun: self.is_pronoun.or(other.is_pronoun),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct AdjectiveData {}

impl AdjectiveData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, _other: &Self) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct AdverbData {}

impl AdverbData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, _other: &Self) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct ConjunctionData {}

impl ConjunctionData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, _other: &Self) -> Self {
        Self {}
    }
}

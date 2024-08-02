use serde::{Deserialize, Serialize};

use super::matcher::Matcher;
use super::Error;

#[derive(Debug, Clone)]
pub struct AffixReplacement {
    pub remove: Vec<char>,
    pub add: Vec<char>,
    pub condition: Matcher
}

impl AffixReplacement {
    pub fn to_human_readable(&self) -> HumanReadableAffixReplacement {
        HumanReadableAffixReplacement {
            remove: self.remove.iter().collect(),
            add: self.add.iter().collect(),
            condition: self.condition.to_string()
        }
    }
}

/// A version of [`AffixReplacement`] that can be serialized to JSON (or
/// whatever) and maintain the nice Regex syntax of the inner [`Matcher`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReadableAffixReplacement {
    pub remove: String,
    pub add: String,
    pub condition: String
}

impl HumanReadableAffixReplacement {
    pub fn to_normal(&self) -> Result<AffixReplacement, Error> {
        Ok(AffixReplacement {
            remove: self.remove.chars().collect(),
            add: self.add.chars().collect(),
            condition: Matcher::parse(&self.condition)?
        })
    }
}

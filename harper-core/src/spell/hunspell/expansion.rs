use serde::{Deserialize, Serialize};

use super::affix_replacement::{AffixReplacement, HumanReadableAffixReplacement};
use super::Error;
use crate::WordMetadata;

#[derive(Debug, Clone)]
pub struct Expansion {
    /// If `!true`, this is a prefix
    pub suffix: bool,
    pub cross_product: bool,
    pub replacements: Vec<AffixReplacement>,
    /// When the expansion is applied, the resulting word will have fields
    /// overwritten with the none-null properties from here.
    pub adds_metadata: WordMetadata
}

impl Expansion {
    pub fn to_human_readable(&self) -> HumanReadableExpansion {
        HumanReadableExpansion {
            suffix: self.suffix,
            cross_product: self.cross_product,
            replacements: self
                .replacements
                .iter()
                .map(AffixReplacement::to_human_readable)
                .collect(),
            adds_metadata: self.adds_metadata
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReadableExpansion {
    pub suffix: bool,
    pub cross_product: bool,
    pub replacements: Vec<HumanReadableAffixReplacement>,
    pub adds_metadata: WordMetadata
}

impl HumanReadableExpansion {
    pub fn to_normal(&self) -> Result<Expansion, Error> {
        let mut replacements = Vec::with_capacity(self.replacements.len());

        for replacement in &self.replacements {
            replacements.push(replacement.to_normal()?);
        }

        Ok(Expansion {
            suffix: self.suffix,
            cross_product: self.cross_product,
            replacements,
            adds_metadata: self.adds_metadata
        })
    }
}

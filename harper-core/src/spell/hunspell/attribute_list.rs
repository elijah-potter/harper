use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use smallvec::{smallvec, ToSmallVec};

use super::affix_replacement::AffixReplacement;
use super::expansion::{Expansion, HumanReadableExpansion};
use super::word_list::MarkedWord;
use super::Error;
use crate::{CharString, Span, WordMetadata};

#[derive(Debug, Clone)]
pub struct AttributeList {
    /// Key = Affix Flag
    affixes: HashMap<char, Expansion>
}

impl AttributeList {
    pub fn to_human_readable(&self) -> HumanReadableAttributeList {
        HumanReadableAttributeList {
            affixes: self
                .affixes
                .iter()
                .map(|(affix, exp)| (*affix, exp.to_human_readable()))
                .collect()
        }
    }

    /// Expand [`MarkedWord`] into a list of full words, including itself.
    ///
    /// Will append to the given `dest`;
    ///
    /// In the future, I want to make this function cleaner and faster.
    pub fn expand_marked_word(
        &self,
        word: MarkedWord,
        metadata: WordMetadata,
        dest: &mut HashMap<CharString, WordMetadata>
    ) {
        dest.reserve(word.attributes.len() + 1);
        let mut gifted_metadata = WordMetadata::default();

        for attr in &word.attributes {
            let Some(expansion) = self.affixes.get(attr) else {
                continue;
            };

            let mut new_words: HashMap<CharString, WordMetadata> = HashMap::new();

            for replacement in &expansion.replacements {
                if let Some(replaced) =
                    Self::apply_replacement(replacement, &word.letters, expansion.suffix)
                {
                    if let Some(val) = new_words.get_mut(&replaced) {
                        *val = val.or(&expansion.adds_metadata);
                    } else {
                        new_words.insert(replaced, expansion.adds_metadata.or(&metadata));
                    }

                    gifted_metadata = gifted_metadata.or(&expansion.gifts_metadata);
                }
            }

            if expansion.cross_product {
                let mut opp_attr = Vec::new();

                for attr in &word.attributes {
                    let Some(attr_def) = self.affixes.get(attr) else {
                        continue;
                    };
                    if attr_def.suffix != expansion.suffix {
                        opp_attr.push(*attr);
                    }
                }

                if new_words.get(&smallvec!['h', 'a', 's']).is_some() {
                    dbg!(&word);
                }

                for (new_word, metadata) in new_words {
                    self.expand_marked_word(
                        MarkedWord {
                            letters: new_word,
                            attributes: opp_attr.clone()
                        },
                        metadata,
                        dest
                    );
                }
            } else {
                for (key, value) in new_words.into_iter() {
                    if let Some(val) = dest.get_mut(&key) {
                        *val = val.or(&value);
                    } else {
                        dest.insert(key, value);
                    }
                }
            }
        }

        if !dest.contains_key(&word.letters) {
            dest.insert(word.letters, metadata.or(&gifted_metadata));
        }
    }

    /// Expand an iterator of marked words into strings.
    /// Note that this does __not__ guarantee that produced words will be
    /// unique.
    pub fn expand_marked_words(
        &self,
        words: impl IntoIterator<Item = (MarkedWord, WordMetadata)>,
        dest: &mut HashMap<CharString, WordMetadata>
    ) {
        for (word, word_metadata) in words {
            self.expand_marked_word(word, word_metadata, dest);
        }
    }

    fn apply_replacement(
        replacement: &AffixReplacement,
        letters: &[char],
        suffix: bool
    ) -> Option<CharString> {
        if replacement.condition.len() > letters.len() {
            return None;
        }

        let target_span = if suffix {
            Span::new(letters.len() - replacement.condition.len(), letters.len())
        } else {
            Span::new(0, replacement.condition.len())
        };

        let target_segment = target_span.get_content(letters);

        if replacement.condition.matches(target_segment) {
            let mut replaced_segment = letters.to_smallvec();
            let mut remove: CharString = replacement.remove.to_smallvec();

            if !suffix {
                replaced_segment.reverse();
            } else {
                remove.reverse();
            }

            for c in &remove {
                let last = replaced_segment.last()?;

                if last == c {
                    replaced_segment.pop();
                } else {
                    return None;
                }
            }

            let mut to_add = replacement.add.to_vec();

            if !suffix {
                to_add.reverse()
            }

            replaced_segment.extend(to_add);

            if !suffix {
                replaced_segment.reverse();
            }

            return Some(replaced_segment);
        }

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReadableAttributeList {
    affixes: HashMap<char, HumanReadableExpansion>
}

impl HumanReadableAttributeList {
    pub fn to_normal(&self) -> Result<AttributeList, Error> {
        let mut affixes = HashMap::with_capacity(self.affixes.len());

        for (affix, expansion) in &self.affixes {
            affixes.insert(*affix, expansion.to_normal()?);
        }

        Ok(AttributeList { affixes })
    }
}

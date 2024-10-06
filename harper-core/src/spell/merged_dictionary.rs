use crate::Lrc;

use hashbrown::HashMap;

use super::dictionary::Dictionary;
use crate::{CharString, WordMetadata};

/// A simple wrapper over [`Dictionary`] that allows
/// one to merge multiple dictionaries without copying.
#[derive(Clone, PartialEq)]
pub struct MergedDictionary<T>
where
    T: Dictionary + Clone,
{
    children: Vec<Lrc<T>>,
    merged: HashMap<CharString, WordMetadata>,
}

impl<T> MergedDictionary<T>
where
    T: Dictionary + Clone,
{
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            merged: HashMap::new(),
        }
    }

    pub fn add_dictionary(&mut self, dictionary: Lrc<T>) {
        self.children.push(dictionary.clone());
    }
}

impl<T> From<Lrc<T>> for MergedDictionary<T>
where
    T: Dictionary + Clone,
{
    fn from(value: Lrc<T>) -> Self {
        Self {
            children: vec![value],
            ..Default::default()
        }
    }
}

impl<T> Default for MergedDictionary<T>
where
    T: Dictionary + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Dictionary for MergedDictionary<T>
where
    T: Dictionary + Clone,
{
    fn contains_word(&self, word: &[char]) -> bool {
        for child in &self.children {
            if child.contains_word(word) {
                return true;
            }
        }
        false
    }

    fn get_word_metadata(&self, word: &[char]) -> WordMetadata {
        let mut found_metadata = WordMetadata::default();
        for child in &self.children {
            found_metadata.append(&child.get_word_metadata(word));
        }

        found_metadata
    }

    fn contains_word_str(&self, word: &str) -> bool {
        let chars: CharString = word.chars().collect();
        self.contains_word(&chars)
    }

    fn get_word_metadata_str(&self, word: &str) -> WordMetadata {
        let chars: CharString = word.chars().collect();
        self.get_word_metadata(&chars)
    }

    fn fuzzy_match(&self, word: &[char], max_distance: u8) -> Vec<(CharString, WordMetadata)> {
        todo!()
    }

    fn fuzzy_match_str(&self, word: &str, max_distance: u8) -> Vec<(CharString, WordMetadata)> {
        todo!()
    }
}

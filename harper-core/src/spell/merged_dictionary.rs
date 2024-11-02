use harper_dictionary_parsing::CharString;
use itertools::Itertools;
use std::sync::Arc;

use super::dictionary::Dictionary;
use crate::WordMetadata;

/// A simple wrapper over [`Dictionary`] that allows
/// one to merge multiple dictionaries without copying.
#[derive(Clone)]
pub struct MergedDictionary {
    children: Vec<Arc<dyn Dictionary>>,
}

impl MergedDictionary {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn add_dictionary(&mut self, dictionary: Arc<dyn Dictionary>) {
        self.children.push(dictionary);
    }
}

impl PartialEq for MergedDictionary {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Default for MergedDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl Dictionary for MergedDictionary {
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

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        Box::new(self.children.iter().flat_map(|c| c.words_iter()))
    }

    fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        Box::new(
            self.children
                .iter()
                .flat_map(move |c| c.words_with_len_iter(len)),
        )
    }

    fn contains_word_str(&self, word: &str) -> bool {
        let chars: CharString = word.chars().collect();
        self.contains_word(&chars)
    }

    fn get_word_metadata_str(&self, word: &str) -> WordMetadata {
        let chars: CharString = word.chars().collect();
        self.get_word_metadata(&chars)
    }

    fn fuzzy_match(
        &self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<(&[char], u8, WordMetadata)> {
        self.children
            .iter()
            .flat_map(|d| d.fuzzy_match(word, max_distance, max_results))
            .sorted_by_key(|r| r.1)
            .take(max_results)
            .collect()
    }

    fn fuzzy_match_str(
        &self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<(&[char], u8, WordMetadata)> {
        self.children
            .iter()
            .flat_map(|d| d.fuzzy_match_str(word, max_distance, max_results))
            .sorted_by_key(|r| r.1)
            .take(max_results)
            .collect()
    }
}

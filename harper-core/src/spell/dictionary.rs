use blanket::blanket;

use crate::{CharString, WordMetadata};

#[cfg(not(feature = "concurrent"))]
#[blanket(derive(Rc))]
pub trait Dictionary {
    /// Check if the dictionary contains a given word.
    fn contains_word(&self, word: &[char]) -> bool;
    /// Check if the dictionary contains a given word.
    fn contains_word_str(&self, word: &str) -> bool;
    /// Gets best fuzzy match from dictionary
    fn fuzzy_match(&self, word: &[char], max_distance: u8) -> Vec<(CharString, WordMetadata)>;
    /// Gets best fuzzy match from dictionary
    fn fuzzy_match_str(&self, word: &str, max_distance: u8) -> Vec<(CharString, WordMetadata)>;
    /// Get the associated [`WordMetadata`] for a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    fn get_word_metadata(&self, word: &[char]) -> WordMetadata;
    /// Get the associated [`WordMetadata`] for a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    fn get_word_metadata_str(&self, word: &str) -> WordMetadata;
}

#[cfg(feature = "concurrent")]
#[blanket(derive(Arc))]
pub trait Dictionary: Send + Sync {
    /// Check if the dictionary contains a given word.
    fn contains_word(&self, word: &[char]) -> bool;
    /// Check if the dictionary contains a given word.
    fn contains_word_str(&self, word: &str) -> bool;
    /// Gets best fuzzy match from dictionary
    fn fuzzy_match(&self, word: &[char], max_distance: u8) -> Vec<(CharString, WordMetadata)>;
    /// Gets best fuzzy match from dictionary
    fn fuzzy_match_str(&self, word: &str, max_distance: u8) -> Vec<(CharString, WordMetadata)>;
    /// Get the associated [`WordMetadata`] for a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    fn get_word_metadata(&self, word: &[char]) -> WordMetadata;
    /// Get the associated [`WordMetadata`] for a given word.
    /// If the word isn't in the dictionary, the resulting metadata will be
    /// empty.
    fn get_word_metadata_str(&self, word: &str) -> WordMetadata;
}

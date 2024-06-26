use hashbrown::HashSet;
use once_cell::sync::Lazy;
use smallvec::{SmallVec, ToSmallVec};

use super::dictionary::Dictionary;
use super::hunspell::{parse_default_attribute_list, parse_default_word_list};
use super::seq_to_normalized;
use crate::CharString;

/// A full, fat dictionary.
/// All of the elements are stored in-memory.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FullDictionary {
    /// Storing a separate [`Vec`] for iterations speeds up spellchecking by
    /// ~16% at the cost of additional memory.
    ///
    /// This is likely due to increased locality 🤷.
    ///
    /// This list is sorted by word length (i.e. the shortest words are first).
    words: Vec<CharString>,
    /// A lookup list for each word length.
    /// Each index of this list will return the first index of [`Self::words`]
    /// that has a word whose index is that length.
    word_len_starts: Vec<usize>,
    /// All English words
    word_set: HashSet<CharString>
}

fn uncached_inner_new() -> FullDictionary {
    let word_list = parse_default_word_list().unwrap();
    let attr_list = parse_default_attribute_list().unwrap();

    // There will be at _least_ this number of words
    let mut words = Vec::with_capacity(word_list.len());

    attr_list.expand_marked_words(word_list, &mut words);

    words.sort();
    words.dedup();

    FullDictionary {
        word_set: HashSet::from_iter(words.iter().cloned()),
        word_len_starts: FullDictionary::create_len_starts(&mut words),
        words
    }
}

static DICT: Lazy<FullDictionary> = Lazy::new(uncached_inner_new);

impl FullDictionary {
    pub fn new() -> Self {
        Self {
            words: Vec::new(),
            word_len_starts: Vec::new(),
            word_set: HashSet::new()
        }
    }

    /// Create a dictionary from the curated Hunspell dictionary included
    /// in the Harper binary.
    pub fn create_from_curated() -> Self {
        DICT.clone()
    }

    /// Appends words to the dictionary.
    /// It is significantly faster to append many words with one call than many
    /// distinct calls to this function.
    pub fn extend_words(&mut self, words: impl IntoIterator<Item = impl AsRef<[char]>>) {
        let init_size = self.words.len();
        self.words
            .extend(words.into_iter().map(|v| v.as_ref().to_smallvec()));
        self.word_set
            .extend(self.words[init_size..].iter().cloned());
        self.word_len_starts = Self::create_len_starts(&mut self.words);
    }

    /// Append a single word to the dictionary.
    ///
    /// If you are appending many words, consider using [`Self::extend_words`]
    /// instead.
    pub fn append_word(&mut self, word: impl AsRef<[char]>) {
        self.extend_words(std::iter::once(word.as_ref()))
    }

    /// Create a lookup table for finding words of a specific length in a word
    /// list. NOTE: This function will sort the original word list by its
    /// length. If the word list's order is changed after creating the
    /// lookup, it will no longer be valid.
    fn create_len_starts(words: &mut [CharString]) -> Vec<usize> {
        words.sort_by_key(|a| a.len());
        let mut word_len_starts = vec![0, 0];

        for (index, len) in words.iter().map(SmallVec::len).enumerate() {
            if word_len_starts.len() <= len {
                word_len_starts.resize(len, index);
                word_len_starts.push(index);
            }
        }

        word_len_starts
    }
}

impl Default for FullDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl Dictionary for FullDictionary {
    fn words_iter(&self) -> impl Iterator<Item = &'_ [char]> {
        self.words.iter().map(|v| v.as_slice())
    }

    /// Iterate over all the words in the dictionary of a given length
    fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + '_> {
        if len == 0 || len >= self.word_len_starts.len() {
            return Box::new(std::iter::empty());
        }

        let start = self.word_len_starts[len];
        let end = if len + 1 == self.word_len_starts.len() {
            self.words.len()
        } else {
            self.word_len_starts[len + 1]
        };

        Box::new(self.words[start..end].iter().map(|v| v.as_slice()))
    }

    fn contains_word(&self, word: &[char]) -> bool {
        let normalized = seq_to_normalized(word);
        let lowercase: SmallVec<_> = normalized.iter().flat_map(|c| c.to_lowercase()).collect();

        self.word_set.contains(normalized.as_ref()) || self.word_set.contains(&lowercase)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{Dictionary, FullDictionary};

    #[test]
    fn curated_contains_no_duplicates() {
        let dict = FullDictionary::create_from_curated();
        assert!(dict.words_iter().all_unique());
    }
}

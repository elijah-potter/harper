use std::{collections::HashSet, iter};

use cached::proc_macro::cached;

#[derive(Debug, Clone)]
pub struct Dictionary {
    /// Storing a separate [Vec] for iterations speeds up spellchecking by ~16% at the cost of
    /// additional memory.
    ///
    /// This is likely due to increased locality :shrug:.
    ///
    /// This list is sorted by word length (i.e. the shortest words are first).
    words: Vec<Vec<char>>,
    /// A lookup list for each word length.
    /// Each index of this list will return the first index of [`Self::words`] that has a word of
    /// that length.
    word_len_starts: Vec<usize>,
    word_set: HashSet<Vec<char>>,
}

#[cached]
fn cached_inner_new() -> Dictionary {
    let english_words_raw = include_str!("../../../english_words.txt").replace('\r', "");

    let mut words: Vec<Vec<char>> = english_words_raw
        .split('\n')
        .filter(|word| !word.is_empty())
        .map(|word| word.chars().collect())
        .collect();

    words.sort_by_key(|a| a.len());

    let mut word_len_starts = vec![0, 0];

    for (index, len) in words.iter().map(Vec::len).enumerate() {
        if word_len_starts.len() == len {
            word_len_starts.push(index);
        }
    }

    Dictionary {
        word_set: HashSet::from_iter(words.iter().cloned()),
        word_len_starts,
        words,
    }
}

impl Dictionary {
    pub fn new() -> Self {
        cached_inner_new()
    }

    /// Iterate over all the words in the dicitonary of a given length
    pub fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + '_> {
        if len == 0 || len >= self.word_len_starts.len() {
            return Box::new(iter::empty());
        }

        let start = self.word_len_starts[len];
        let end = if len + 1 == self.word_len_starts.len() {
            self.words.len()
        } else {
            self.word_len_starts[len + 1]
        };

        Box::new(self.words[start..end].iter().map(|v| v.as_slice()))
    }

    pub fn words_iter(&self) -> impl Iterator<Item = &'_ [char]> {
        self.words.iter().map(|v| v.as_slice())
    }

    pub fn contains_word(&self, word: &[char]) -> bool {
        let lowercase: Vec<_> = word.iter().flat_map(|c| c.to_lowercase()).collect();

        self.word_set.contains(word) || self.word_set.contains(&lowercase)
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

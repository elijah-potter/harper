use hashbrown::HashSet;
use once_cell::sync::Lazy;
use smallvec::SmallVec;

use super::{
    hunspell::{parse_default_attribute_list, parse_default_word_list},
    DictWord,
};

#[derive(Debug, Clone)]
pub struct Dictionary {
    /// Storing a separate [Vec] for iterations speeds up spellchecking by ~16% at the cost of
    /// additional memory.
    ///
    /// This is likely due to increased locality :shrug:.
    ///
    /// This list is sorted by word length (i.e. the shortest words are first).
    words: Vec<DictWord>,
    /// A lookup list for each word length.
    /// Each index of this list will return the first index of [`Self::words`] that has a word
    /// whose index is that length.
    word_len_starts: Vec<usize>,
    word_set: HashSet<DictWord>,
}

fn uncached_inner_new() -> Dictionary {
    let word_list = parse_default_word_list().unwrap();
    let attr_list = parse_default_attribute_list().unwrap();

    let words = attr_list.expand_marked_words(word_list).unwrap();
    let mut words: Vec<DictWord> = words.into_iter().collect();

    words.sort_by_key(|a| a.len());

    let mut word_len_starts = vec![0, 0];

    for (index, len) in words.iter().map(SmallVec::len).enumerate() {
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

static DICT: Lazy<Dictionary> = Lazy::new(uncached_inner_new);

impl Dictionary {
    pub fn new() -> Self {
        DICT.clone()
    }

    /// Iterate over all the words in the dicitonary of a given length
    pub fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + '_> {
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

    pub fn words_iter(&self) -> impl Iterator<Item = &'_ [char]> {
        self.words.iter().map(|v| v.as_slice())
    }

    pub fn contains_word(&self, word: &[char]) -> bool {
        let lowercase: SmallVec<_> = word.iter().flat_map(|c| c.to_lowercase()).collect();

        self.word_set.contains(word) || self.word_set.contains(&lowercase)
    }
}

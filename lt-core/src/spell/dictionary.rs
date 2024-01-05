use std::collections::HashSet;

pub struct Dictionary {
    /// Storing a separate [Vec] for iterations speeds up spellchecking by ~16% at the cost of
    /// additional memory.
    ///
    /// This is likely due to increased locality :shrug:.
    words: Vec<Vec<char>>,
    word_set: HashSet<Vec<char>>,
}

impl Dictionary {
    pub fn create_from_static() -> Self {
        let english_words_raw = include_str!("../../../english_words.txt").replace('\r', "");

        let words: Vec<_> = english_words_raw
            .split('\n')
            .filter(|word| !word.is_empty())
            .map(|word| word.chars().collect())
            .collect();

        Self {
            word_set: HashSet::from_iter(words.iter().cloned()),
            words,
        }
    }

    pub fn words_iter(&self) -> impl Iterator<Item = &'_ [char]> {
        self.words.iter().map(|v| v.as_slice())
    }

    pub fn contains_word(&self, word: &[char]) -> bool {
        let lowercase: Vec<_> = word.iter().flat_map(|c| c.to_lowercase()).collect();

        self.word_set.contains(word) || self.word_set.contains(&lowercase)
    }
}

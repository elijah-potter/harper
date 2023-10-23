use std::collections::HashSet;

pub struct Dictionary {
    /// Storing a separate [Vec] for iterations speeds up spellchecking by ~16% at the cost of
    /// additional memory.
    words: Vec<Vec<char>>,
    word_set: HashSet<Vec<char>>,
}

impl Dictionary {
    pub fn create_from_static() -> Self {
        let english_words_raw = include_str!("../../../english_words.txt").replace('\r', "");

        Self {
            word_set: english_words_raw
                .split('\n')
                .filter(|word| !word.is_empty())
                .map(|word| word.chars().collect())
                .collect(),
            words: english_words_raw
                .split('\n')
                .filter(|word| !word.is_empty())
                .map(|word| word.chars().collect())
                .collect(),
        }
    }

    pub fn words_iter(&self) -> impl Iterator<Item = &'_ [char]> {
        self.words.iter().map(|v| v.as_slice())
    }

    pub fn contains_word(&self, word: &[char]) -> bool {
        self.word_set.contains(word)
    }
}

use hashbrown::HashMap;
use smallvec::{SmallVec, ToSmallVec};

use super::dictionary::Dictionary;
use super::hunspell::{parse_default_attribute_list, parse_default_word_list};
use super::seq_to_normalized;
use crate::{CharString, Lrc, WordMetadata};

/// A full, fat dictionary.
/// All elements are stored in-memory.
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
    word_map: HashMap<CharString, WordMetadata>,
}

/// The uncached function that is used to produce the original copy of the
/// curated dictionary.
fn uncached_inner_new() -> Lrc<FullDictionary> {
    let word_list = parse_default_word_list().unwrap();
    let attr_list = parse_default_attribute_list();

    // There will be at _least_ this number of words
    let mut word_map = HashMap::with_capacity(word_list.len());

    attr_list.expand_marked_words(word_list, &mut word_map);

    let mut words: Vec<CharString> = word_map.iter().map(|(v, _)| v.clone()).collect();
    words.sort();
    words.dedup();

    Lrc::new(FullDictionary {
        word_map,
        word_len_starts: FullDictionary::create_len_starts(&mut words),
        words,
    })
}

thread_local! {
    static DICT: Lrc<FullDictionary> = uncached_inner_new();
}

impl FullDictionary {
    pub fn new() -> Self {
        Self {
            words: Vec::new(),
            word_len_starts: Vec::new(),
            word_map: HashMap::new(),
        }
    }

    /// Create a dictionary from the curated dictionary included
    /// in the Harper binary.
    pub fn curated() -> Lrc<Self> {
        DICT.with(|v| v.clone())
    }

    /// Appends words to the dictionary.
    /// It is significantly faster to append many words with one call than many
    /// distinct calls to this function.
    pub fn extend_words(
        &mut self,
        words: impl IntoIterator<Item = (impl AsRef<[char]>, WordMetadata)>,
    ) {
        let pairs: Vec<_> = words
            .into_iter()
            .map(|(v, m)| (v.as_ref().to_smallvec(), m))
            .collect();

        self.words.extend(pairs.iter().map(|(v, _)| v.clone()));
        self.word_len_starts = Self::create_len_starts(&mut self.words);
        self.word_map.extend(pairs);
    }

    /// Append a single word to the dictionary.
    ///
    /// If you are appending many words, consider using [`Self::extend_words`]
    /// instead.
    pub fn append_word(&mut self, word: impl AsRef<[char]>, metadata: WordMetadata) {
        self.extend_words(std::iter::once((word.as_ref(), metadata)))
    }

    /// Append a single string to the dictionary.
    ///
    /// If you are appending many words, consider using [`Self::extend_words`]
    /// instead.
    pub fn append_word_str(&mut self, word: &str, metadata: WordMetadata) {
        self.append_word(word.chars().collect::<Vec<_>>(), metadata)
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

    fn get_word_metadata(&self, word: &[char]) -> WordMetadata {
        let normalized = seq_to_normalized(word);
        let lowercase: CharString = normalized.iter().flat_map(|c| c.to_lowercase()).collect();

        self.word_map
            .get(normalized.as_ref())
            .cloned()
            .or(self.word_map.get(lowercase.as_ref()).cloned())
            .unwrap_or(WordMetadata::default())
    }

    fn contains_word(&self, word: &[char]) -> bool {
        let normalized = seq_to_normalized(word);
        let lowercase: CharString = normalized.iter().flat_map(|c| c.to_lowercase()).collect();

        self.word_map.contains_key(normalized.as_ref()) || self.word_map.contains_key(&lowercase)
    }

    fn contains_word_str(&self, word: &str) -> bool {
        let chars: CharString = word.chars().collect();
        self.contains_word(&chars)
    }

    fn get_word_metadata_str(&self, word: &str) -> WordMetadata {
        let chars: CharString = word.chars().collect();
        self.get_word_metadata(&chars)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{Dictionary, FullDictionary};

    #[test]
    fn curated_contains_no_duplicates() {
        let dict = FullDictionary::curated();
        assert!(dict.words_iter().all_unique());
    }

    #[test]
    fn curated_matches_capitalized() {
        let dict = FullDictionary::curated();
        assert!(dict.contains_word_str("this"));
        assert!(dict.contains_word_str("This"));
    }

    #[test]
    fn this_is_noun() {
        let dict = FullDictionary::curated();
        assert!(dict.get_word_metadata_str("this").is_noun());
        assert!(dict.get_word_metadata_str("This").is_noun());
    }

    #[test]
    fn than_is_conjunction() {
        let dict = FullDictionary::curated();
        assert!(dict.get_word_metadata_str("than").is_conjunction());
        assert!(dict.get_word_metadata_str("Than").is_conjunction());
    }

    #[test]
    fn herself_is_pronoun() {
        let dict = FullDictionary::curated();
        assert!(dict.get_word_metadata_str("than").is_conjunction());
        assert!(dict.get_word_metadata_str("Than").is_conjunction());
    }

    #[test]
    fn discussion_171() {
        let dict = FullDictionary::curated();
        assert!(dict.contains_word_str("natively"));
    }

    #[test]
    fn im_is_common() {
        let dict = FullDictionary::curated();
        assert!(dict.get_word_metadata_str("I'm").common);
    }
}

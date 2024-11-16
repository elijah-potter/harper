use super::{
    edit_distance_min_alloc,
    hunspell::{parse_default_attribute_list, parse_default_word_list},
    seq_to_normalized,
};
use hashbrown::HashMap;
use itertools::Itertools;
use smallvec::{SmallVec, ToSmallVec};
use std::sync::Arc;

use crate::{CharString, CharStringExt, WordMetadata};

use super::dictionary::Dictionary;
use super::FuzzyMatchResult;

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
fn uncached_inner_new() -> Arc<FullDictionary> {
    let word_list = parse_default_word_list().unwrap();
    let attr_list = parse_default_attribute_list();

    // There will be at _least_ this number of words
    let mut word_map = HashMap::with_capacity(word_list.len());

    attr_list.expand_marked_words(word_list, &mut word_map);

    let mut words: Vec<CharString> = word_map.iter().map(|(v, _)| v.clone()).collect();
    // This may seem weird, but it ensures that the indexes in the Words vector match up with those
    // stored in the computed FST Map.
    words.sort_unstable();
    words.dedup();
    words.sort_by_key(|w| w.len()); // DO NOT MAKE THIS UNSTABLE.

    Arc::new(FullDictionary {
        word_map,
        word_len_starts: FullDictionary::create_len_starts(&words),
        words,
    })
}

thread_local! {
    static DICT: Arc<FullDictionary> = uncached_inner_new();
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
    /// Consider using [`super::FstDictionary::curated()`] instead, as it is more performant for spellchecking.
    pub fn curated() -> Arc<Self> {
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
        self.words.sort_by_key(|w| w.len());
        self.word_len_starts = Self::create_len_starts(&self.words);
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
    fn create_len_starts(words: &[CharString]) -> Vec<usize> {
        let mut len_words: Vec<_> = words.to_vec();
        len_words.sort_by_key(|a| a.len());

        let mut word_len_starts = vec![0, 0];

        for (index, len) in len_words.iter().map(SmallVec::len).enumerate() {
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
    fn get_word_metadata(&self, word: &[char]) -> WordMetadata {
        let normalized = seq_to_normalized(word);
        let lowercase: CharString = normalized.to_lower();

        self.word_map
            .get(normalized.as_ref())
            .cloned()
            .or(self.word_map.get(lowercase.as_ref()).cloned())
            .unwrap_or(WordMetadata::default())
    }

    fn contains_word(&self, word: &[char]) -> bool {
        let normalized = seq_to_normalized(word);
        let lowercase: CharString = normalized.to_lower();

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

    /// Suggest a correct spelling for a given misspelled word.
    /// `Self::word` is assumed to be quite small (n < 100).
    /// `max_distance` relates to an optimization that allows the search
    /// algorithm to prune large portions of the search.
    fn fuzzy_match(
        &self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult> {
        let misspelled_charslice = seq_to_normalized(word);

        let shortest_word_len = if misspelled_charslice.len() <= max_distance as usize {
            1
        } else {
            misspelled_charslice.len() - max_distance as usize
        };
        let longest_word_len = misspelled_charslice.len() + max_distance as usize;

        // Get candidate words
        let words_to_search = (shortest_word_len..=longest_word_len)
            .rev()
            .flat_map(|len| self.words_with_len_iter(len));

        // Pre-allocated vectors for the edit-distance calculation
        // 53 is the length of the longest word.
        let mut buf_a = Vec::with_capacity(53);
        let mut buf_b = Vec::with_capacity(53);

        // Sort by edit-distance
        words_to_search
            .filter_map(|word| {
                let dist =
                    edit_distance_min_alloc(&misspelled_charslice, word, &mut buf_a, &mut buf_b);

                if dist <= max_distance {
                    Some((word, dist))
                } else {
                    None
                }
            })
            .sorted_unstable_by_key(|a| a.1)
            .take(max_results)
            .map(|(word, edit_distance)| FuzzyMatchResult {
                word,
                edit_distance,
                metadata: self.get_word_metadata(word),
            })
            .collect()
    }

    fn fuzzy_match_str(
        &self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult> {
        let word: Vec<_> = word.chars().collect();
        self.fuzzy_match(&word, max_distance, max_results)
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        Box::new(self.words.iter().map(|v| v.as_slice()))
    }

    fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
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
}

#[cfg(test)]
mod tests {
    use crate::CharString;
    use itertools::Itertools;

    use crate::{Dictionary, FullDictionary};

    #[test]
    fn words_with_len_contains_self() {
        let dict = FullDictionary::curated();

        let word: CharString = "hello".chars().collect();
        let words_with_same_len = dict.words_with_len_iter(word.len()).collect_vec();
        assert!(words_with_same_len.contains(&&word[..]));
    }

    #[test]
    fn curated_contains_no_duplicates() {
        let dict = FullDictionary::curated();
        assert!(dict.words.iter().all_unique());
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
        assert!(dict.get_word_metadata_str("herself").is_pronoun());
        assert!(dict.get_word_metadata_str("Herself").is_pronoun());
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

    #[test]
    fn fuzzy_result_sorted_by_edit_distance() {
        let dict = FullDictionary::curated();

        let results = dict.fuzzy_match_str("hello", 3, 100);
        let is_sorted_by_dist = results
            .iter()
            .map(|fm| fm.edit_distance)
            .tuple_windows()
            .all(|(a, b)| a <= b);

        assert!(is_sorted_by_dist)
    }
}

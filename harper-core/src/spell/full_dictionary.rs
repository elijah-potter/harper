use super::{edit_distance_min_alloc, seq_to_normalized};
use hashbrown::HashMap;
use smallvec::{SmallVec, ToSmallVec};

use super::dictionary::Dictionary;
use crate::{CharString, CharStringExt, Lrc, WordMetadata};
use harper_dictionary_parsing::{parse_default_attribute_list, parse_default_word_list};

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
        word_len_starts: FullDictionary::create_len_starts(&words),
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

    pub fn get_word(&self, index: usize) -> &CharString {
        &self.words[index]
    }
    pub fn get_metadata(&self, index: usize) -> &WordMetadata {
        self.word_map.get(self.get_word(index)).unwrap()
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
    /// [`word`] is assumed to be quite small (n < 100).
    /// [`max_distance`] relates to an optimization that allows the search
    /// algorithm to prune large portions of the search.
    fn fuzzy_match(
        &self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<(&[char], u8, WordMetadata)> {
        let misspelled_normalized = seq_to_normalized(word);
        let misspelled_lower: Vec<char> = misspelled_normalized.to_lower().to_vec();

        // The length of the shortest word to look at.
        let shortest_word_len = if misspelled_normalized.len() < max_distance as usize {
            1
        } else {
            misspelled_normalized.len() - max_distance as usize
        };

        // Note how we look at the biggest words first.
        let words_to_search = (shortest_word_len
            ..misspelled_normalized.len() + max_distance as usize)
            .rev()
            .flat_map(|len| self.words_with_len_iter(len));

        // Pre-allocated vectors for the edit-distance calculation
        // 53 is the length of the longest word.
        let mut buf_a = Vec::with_capacity(53);
        let mut buf_b = Vec::with_capacity(53);
        let pruned_words = words_to_search.filter_map(|word| {
            let dist =
                edit_distance_min_alloc(&misspelled_normalized, word, &mut buf_a, &mut buf_b);
            let dist_lower =
                edit_distance_min_alloc(&misspelled_lower, word, &mut buf_a, &mut buf_b);
            let smallest_dist = std::cmp::min(dist, dist_lower);

            if smallest_dist <= max_distance {
                Some((word, smallest_dist))
            } else {
                None
            }
        });

        // Locate the words with the lowest edit distance.
        let mut found_dist: Vec<(&[char], u8)> = Vec::with_capacity(max_results);
        for (word, dist) in pruned_words {
            if found_dist.len() < max_results {
                found_dist.push((word, dist));
            } else if dist < found_dist[max_results - 1].1 {
                found_dist[max_results - 1] = (word, dist);
            }
            found_dist.sort_by_key(|a| a.1);
        }

        // Create final, ordered list of suggestions.
        found_dist
            .into_iter()
            .map(|(word, dist)| (word, dist, self.get_word_metadata(word)))
            .collect()
    }

    fn fuzzy_match_str(
        &self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<(&[char], u8, WordMetadata)> {
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
    use itertools::Itertools;

    use super::super::{edit_distance, seq_to_normalized};
    use crate::{Dictionary, FullDictionary};

    fn assert_edit_dist(source: &str, target: &str, expected: u8) {
        let source: Vec<_> = source.chars().collect();
        let target: Vec<_> = target.chars().collect();

        let dist = edit_distance(&source, &target);
        assert_eq!(dist, expected)
    }

    #[test]
    fn normalizes_weve() {
        let word = vec!['w', 'e', '’', 'v', 'e'];
        let norm = seq_to_normalized(&word);

        assert_eq!(norm.clone(), vec!['w', 'e', '\'', 'v', 'e'])
    }

    #[test]
    fn simple1() {
        assert_edit_dist("kitten", "sitting", 3)
    }

    #[test]
    fn simple2() {
        assert_edit_dist("saturday", "sunday", 3)
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
}

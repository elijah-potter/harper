use super::{edit_distance_min_alloc, seq_to_normalized, FullDictionary};
use fst::Map as FstMap;
use fst::{automaton::Levenshtein, IntoStreamer};
use itertools::Itertools;

use crate::{CharStringExt, Lrc, WordMetadata};

use super::Dictionary;

#[derive(Debug)]
pub struct FstDictionary {
    /// Underlying FullDictionary used for everything except fuzzy finding
    full_dict: Lrc<FullDictionary>,
    /// Used for fuzzy-finding the index of words or metadata
    word_map: FstMap<Vec<u8>>,
}

/// The uncached function that is used to produce the original copy of the
/// curated dictionary.
fn uncached_inner_new() -> Lrc<FstDictionary> {
    let full_dict = FullDictionary::curated();
    let word_map = FstMap::new(include_bytes!("../../dictionary.fst").to_vec()).unwrap();

    Lrc::new(FstDictionary {
        full_dict,
        word_map,
    })
}

thread_local! {
    static DICT: Lrc<FstDictionary> = uncached_inner_new();
}

impl PartialEq for FstDictionary {
    fn eq(&self, other: &Self) -> bool {
        self.full_dict == other.full_dict
    }
}

impl FstDictionary {
    /// Create a dictionary from the curated dictionary included
    /// in the Harper binary.
    pub fn curated() -> Lrc<Self> {
        DICT.with(|v| v.clone())
    }
}

impl Dictionary for FstDictionary {
    fn contains_word(&self, word: &[char]) -> bool {
        self.full_dict.contains_word(word)
    }

    fn contains_word_str(&self, word: &str) -> bool {
        self.full_dict.contains_word_str(word)
    }

    fn get_word_metadata(&self, word: &[char]) -> WordMetadata {
        self.full_dict.get_word_metadata(word)
    }

    fn get_word_metadata_str(&self, word: &str) -> WordMetadata {
        self.full_dict.get_word_metadata_str(word)
    }

    fn fuzzy_match(
        &self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<(&[char], u8, WordMetadata)> {
        self.fuzzy_match_str(&word.iter().collect::<String>(), max_distance, max_results)
    }

    fn fuzzy_match_str(
        &self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<(&[char], u8, WordMetadata)> {
        // Various transformations of the input
        let chars: Vec<_> = word.chars().collect();
        let misspelled_word_charslice = seq_to_normalized(&chars);
        let misspelled_lower_charslice = misspelled_word_charslice.to_lower();
        let misspelled_word_string = misspelled_word_charslice.to_string();

        // Actual FST search
        let automaton = Levenshtein::new(&misspelled_word_string, max_distance as u32).unwrap();
        let word_indexes_stream = self.word_map.search(automaton).into_stream().into_values();

        // Pre-allocated vectors for edit-distance calculation
        // 53 is the length of the longest word.
        let mut buf_a = Vec::with_capacity(53);
        let mut buf_b = Vec::with_capacity(53);

        word_indexes_stream
            .into_iter()
            .sorted_unstable()
            .dedup()
            .map(|index| (self.full_dict.get_word(index as usize), index))
            // Sort by edit distance
            .map(|(word, index)| {
                let dist = edit_distance_min_alloc(
                    &misspelled_lower_charslice,
                    &word.to_lower(),
                    &mut buf_a,
                    &mut buf_b,
                );

                (word, dist, index)
            })
            .sorted_unstable_by_key(|a| a.1)
            .take(max_results)
            .map(|(word, dist, index)| {
                (
                    word.as_slice(),
                    dist,
                    self.full_dict.get_metadata(index as usize).to_owned(),
                )
            })
            .collect()
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        self.full_dict.words_iter()
    }

    fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        self.full_dict.words_with_len_iter(len)
    }
}

#[cfg(test)]
mod tests {
    use fst::IntoStreamer;
    use itertools::Itertools;

    use crate::{spell::seq_to_normalized, CharStringExt, Dictionary};

    use super::FstDictionary;

    #[test]
    fn fst_map_contains_all_in_full_dict() {
        let dict = FstDictionary::curated();

        for word in dict.words_iter() {
            let misspelled_normalized = seq_to_normalized(word);
            let misspelled_word: String = misspelled_normalized.to_string();
            let misspelled_lower: String = misspelled_normalized.to_lower().to_string();

            assert!(!misspelled_word.is_empty());
            assert!(
                dict.word_map.contains_key(misspelled_word)
                    || dict.word_map.contains_key(misspelled_lower)
            );
        }
    }

    #[test]
    fn fst_contains_hello() {
        let dict = FstDictionary::curated();

        let word: Vec<_> = "hello".chars().collect();
        let misspelled_normalized = seq_to_normalized(&word);
        let misspelled_word: String = misspelled_normalized.to_string();
        let misspelled_lower: String = misspelled_normalized.to_lower().to_string();

        assert!(dict.contains_word(&misspelled_normalized));
        assert!(
            dict.word_map.contains_key(misspelled_lower)
                || dict.word_map.contains_key(misspelled_word)
        );
    }

    #[test]
    fn fst_search_hello() {
        let dict = FstDictionary::curated();

        let word: Vec<_> = "hvllo".chars().collect();
        let misspelled_normalized = seq_to_normalized(&word);
        let misspelled_word: String = misspelled_normalized.to_string();
        let misspelled_lower: String = misspelled_normalized.to_lower().to_string();

        let aut = fst::automaton::Levenshtein::new(&misspelled_word, 2).unwrap();
        let aut_lower = fst::automaton::Levenshtein::new(&misspelled_lower, 2).unwrap();
        let word_indexes_stream = dict
            .word_map
            .search(aut)
            .into_stream()
            .into_str_keys()
            .unwrap();
        let word_lower_indexes_stream = dict
            .word_map
            .search(aut_lower)
            .into_stream()
            .into_str_keys()
            .unwrap();

        dbg!(&word_indexes_stream);
        assert!(
            word_indexes_stream.contains(&"hello".to_string())
                || word_lower_indexes_stream.contains(&"hello".to_string())
        );
    }

    #[test]
    fn fuzzy_result_sorted_by_edit_distance() {
        let dict = FstDictionary::curated();

        let results = dict.fuzzy_match_str("hello", 3, 100);
        let is_sorted_by_dist = results
            .iter()
            .map(|(_, dist, _)| dist)
            .tuple_windows()
            .all(|(a, b)| a <= b);

        assert!(is_sorted_by_dist)
    }
}

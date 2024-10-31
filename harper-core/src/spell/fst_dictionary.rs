use super::{seq_to_normalized, FullDictionary};
use fst::{automaton::Levenshtein, IntoStreamer};
use fst::{Map as FstMap, Streamer};

use crate::{Lrc, WordMetadata};

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
        let chars: Vec<_> = word.chars().collect();
        let misspelled_word = seq_to_normalized(&chars);
        let misspelled_lower: String = misspelled_word
            .iter()
            .flat_map(|v| v.to_lowercase())
            .collect();

        let aut = Levenshtein::new(&misspelled_lower, max_distance as u32).unwrap();
        let mut word_indexes_stream = self.word_map.search(aut).into_stream();
        let mut word_indexes = Vec::with_capacity(max_results);

        let mut i = 0;
        while i < max_results {
            if let Some(v) = word_indexes_stream.next() {
                word_indexes.push(v.1);
            } else {
                break;
            }
            i += 1;
        }
        word_indexes
            .into_iter()
            .take(max_results)
            .map(|i| (self.full_dict.get_word(i as usize), i))
            .map(|(word, i)| {
                (
                    word.as_slice(),
                    i as u8,
                    self.full_dict.get_metadata(i as usize).to_owned(),
                )
            })
            .collect()
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + '_> {
        self.full_dict.words_iter()
    }

    fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + '_> {
        self.full_dict.words_with_len_iter(len)
    }
}

#[cfg(test)]
mod tests {
    use fst::IntoStreamer;

    use crate::{spell::seq_to_normalized, Dictionary};

    use super::FstDictionary;

    #[test]
    fn fst_contains() {
        let dict = FstDictionary::curated();

        for word in dict.words_iter() {
            let misspelled_word = seq_to_normalized(word);
            let misspelled_lower: String = misspelled_word
                .iter()
                .flat_map(|v| v.to_lowercase())
                .collect();

            println!("{}", misspelled_lower);
            assert!(!misspelled_lower.is_empty());
            assert!(dict.contains_word(word));
            assert!(dict.word_map.contains_key(misspelled_lower));
        }
    }

    #[test]
    fn fst_words_match() {
        let dict = FstDictionary::curated();

        for (word, i) in dict.word_map.into_stream().into_str_vec().unwrap() {
            let full_dict_word = dict
                .full_dict
                .get_word(i as usize)
                .iter()
                .collect::<String>();
            println!("\"{}\" == \"{}\"?", word, full_dict_word);
            assert_eq!(word, full_dict_word);
        }
    }

    #[test]
    fn fst_contains_hello() {
        let dict = FstDictionary::curated();

        let word: Vec<_> = "hello".chars().collect();
        let misspelled_word = seq_to_normalized(&word);
        let misspelled_lower: String = misspelled_word
            .iter()
            .flat_map(|v| v.to_lowercase())
            .collect();

        assert!(dict.contains_word(&word));
        assert!(dict.word_map.contains_key(misspelled_lower));
    }
}

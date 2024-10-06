use std::fs::File;

use fst::Map as FstMap;
use fst::{automaton::Levenshtein, IntoStreamer};
use memmap::Mmap;

use super::hunspell::{parse_default_attribute_list, parse_default_word_list};
use crate::{CharString, Lrc, WordMetadata};

use super::Dictionary;

#[derive(Debug)]
pub struct FstDictionary {
    /// Sorted by string in lexicographic order
    metadata: Vec<WordMetadata>,
    /// Used for fuzzy-finding the index of words or metadata
    word_map: FstMap<Mmap>,
}

/// The uncached function that is used to produce the original copy of the
/// curated dictionary.
fn uncached_inner_new() -> Lrc<FstDictionary> {
    let word_list = parse_default_word_list().unwrap();
    let attr_list = parse_default_attribute_list();

    // There will be at _least_ this number of words
    // This creates a memory map, which enables searching the map without loading
    // all of it into memory.
    let mmap = unsafe { Mmap::map(&File::open("../../dictionary.fst").unwrap()).unwrap() };
    let word_map = FstMap::new(mmap).unwrap();

    let mut words: Vec<CharString> = word_list.iter().map(|mw| mw.letters).collect();
    words.sort();
    words.dedup();

    Lrc::new(FstDictionary { metadata, word_map })
}

thread_local! {
    static DICT: Lrc<FstDictionary> = uncached_inner_new();
}

impl Dictionary for FstDictionary {
    fn contains_word(&self, word: &[char]) -> bool {
        self.word_map.contains_key(word.iter().collect::<String>())
    }

    fn contains_word_str(&self, word: &str) -> bool {
        self.word_map.contains_key(word)
    }

    fn get_word_metadata(&self, word: &[char]) -> WordMetadata {
        let index: usize = self.word_map.get(word.iter().collect::<String>()).unwrap() as usize;
        self.metadata[index]
    }

    fn get_word_metadata_str(&self, word: &str) -> WordMetadata {
        let index: usize = self.word_map.get(word).unwrap() as usize;
        self.metadata[index]
    }

    fn fuzzy_match(&self, word: &[char], max_distance: u8) -> Vec<(CharString, WordMetadata)> {
        let aut = Levenshtein::new(&word.iter().collect::<String>(), max_distance as u32).unwrap();
        let words = self
            .word_map
            .search(aut)
            .into_stream()
            .into_str_vec()
            .unwrap();
        words
            .into_iter()
            .map(|(word, i)| (word.chars().collect(), self.metadata[i as usize]))
            .collect()
    }

    fn fuzzy_match_str(&self, word: &str, max_distance: u8) -> Vec<(CharString, WordMetadata)> {
        let aut = Levenshtein::new(word, max_distance as u32).unwrap();
        let words = self
            .word_map
            .search(aut)
            .into_stream()
            .into_str_vec()
            .unwrap();
        words
            .into_iter()
            .map(|(word, i)| (word.chars().collect(), self.metadata[i as usize]))
            .collect()
    }
}

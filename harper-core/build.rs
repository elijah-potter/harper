use std::fs::File;
use std::io;

use fst::MapBuilder;
use harper_dictionary_parsing::{
    parse_default_attribute_list, parse_default_word_list, CharString,
};
use hashbrown::HashMap;
use itertools::Itertools;

fn main() {
    let wtr = io::BufWriter::new(File::create("dictionary.fst").unwrap());
    let mut build = MapBuilder::new(wtr).unwrap();

    let word_list = parse_default_word_list().unwrap();
    let attr_list = parse_default_attribute_list();

    // There will be at _least_ this number of words
    let mut word_map = HashMap::with_capacity(word_list.len());

    attr_list.expand_marked_words(word_list, &mut word_map);

    let mut words: Vec<CharString> = word_map.iter().map(|(w, _)| w.clone()).collect();
    // This may seem weird, but it ensures that the indexes from enumerate below match with those
    // of the words array in FullDictionary.
    words.sort_unstable();
    words.dedup();
    words.sort_by_key(|w| w.len()); // DO NOT MAKE THIS UNSTABLE.

    words
        .iter()
        .enumerate()
        .sorted_by_key(|w| w.1)
        .for_each(|(i, s)| {
            let word = s.iter().collect::<String>();
            build
                .insert(word, i as u64)
                .expect("Insertion not in lexicographical order!");
        });

    build.finish().expect("Unable to build map of dictionary!");

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../harper-dictionary-parsing/dictionary.dict");
}

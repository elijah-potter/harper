use std::fs::File;
use std::io;

use fst::MapBuilder;

pub fn parse_word_list(source: &str) -> Vec<String> {
    let mut lines = source.lines();

    let approx_item_count = lines.next().unwrap().parse().unwrap();
    let mut words = Vec::with_capacity(approx_item_count);

    for line in lines {
        if let Some((word, _attributes)) = line.split_once('/') {
            words.push(word.chars().collect())
        } else {
            words.push(line.chars().collect())
        }
    }

    words
}

fn main() {
    let mut wtr = io::BufWriter::new(File::create("dictionary.fst").unwrap());
    let mut build = MapBuilder::new(wtr).unwrap();

    let mut word_list: Vec<String> = parse_word_list(include_str!("dictionary.dict"));
    word_list.sort();
    word_list.dedup();

    // Using u64 shouldn't pose any issues since I don't think the English
    // language has that many words
    word_list
        .iter()
        .enumerate()
        .for_each(|(i, s)| build.insert(s, i as u64).unwrap());

    build.finish().expect("Unable to build map of dictionary!");

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=dictionary.dict");
}

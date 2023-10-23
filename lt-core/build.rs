use regex::Regex;
use std::io::Write;
use std::{env, fs::File, io::BufWriter, path::Path};

// Generate the code needed to embed the word list as a static array to be included in the binary.
fn main() {
    let english_words_raw = include_str!("../english_words.txt").replace('\r', "");

    let re = Regex::new(r"[\r-']").unwrap();
    let cleaned = re.replace_all(&english_words_raw, "");

    let words: Vec<_> = cleaned
        .split('\n')
        .filter(|word| !word.is_empty())
        .collect();

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("english.rs");
    let mut out_file = BufWriter::new(File::create(path).unwrap());

    write!(
        out_file,
        "#[allow(clippy::all)] static ENGLISH_WORDS: [&[char]; {}] = [",
        words.len()
    )
    .unwrap();

    for word in words {
        write!(out_file, "&[").unwrap();
        for character in word.chars() {
            write!(out_file, "'{}', ", character).unwrap();
        }
        write!(out_file, "],").unwrap();
    }

    write!(out_file, "];").unwrap();
}

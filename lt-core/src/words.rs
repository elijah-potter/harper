include!(concat!(env!("OUT_DIR"), "/english.rs"));

pub fn english_words() -> &'static [&'static [char]] {
    &ENGLISH_WORDS
}

mod affix_replacement;
mod attribute_list;
mod error;
mod expansion;
mod matcher;
mod word_list;

pub use attribute_list::AttributeList;
pub use error::Error;

use self::word_list::parse_word_list;
pub use self::word_list::MarkedWord;

pub fn parse_default_word_list() -> Result<Vec<MarkedWord>, Error> {
    parse_word_list(include_str!("../../../dictionary.dict"))
}

pub fn parse_default_attribute_list() -> Result<AttributeList, Error> {
    AttributeList::parse(include_str!("../../../dictionary.aff"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::attribute_list::AttributeList;
    use super::word_list::parse_word_list;
    use super::{parse_default_attribute_list, parse_default_word_list};
    use crate::CharString;

    pub const TEST_WORD_LIST: &str = "3\nhello\ntry/B\nwork/AB";
    pub const ATTR_LIST: &str =
        "SET UTF-8\nTRY esianrtolcdugmphbyfvkwzESIANRTOLCDUGMPHBYFVKWZ'\n\nREP 2\nREP f ph\nREP \
         ph f\n\nPFX A Y 1\nPFX A 0 re .\n\nSFX B Y 2\nSFX B 0 ed [^y]\nSFX B y ied y";

    #[test]
    fn dump() {
        let default = parse_default_attribute_list().unwrap();
        let default_human_readable = default.to_human_readable();

        let dumped = serde_json::to_string_pretty(&default_human_readable).unwrap();
        fs::write("affixes.json", dumped.as_bytes());
    }

    #[test]
    fn correctly_expands_test_files() {
        let words = parse_word_list(TEST_WORD_LIST).unwrap();
        let attributes = AttributeList::parse(ATTR_LIST).unwrap();

        let mut expanded = Vec::new();

        attributes.expand_marked_words(words, &mut expanded);
        let expanded: Vec<String> = expanded
            .into_iter()
            .map(|v| v.into_iter().collect())
            .collect();

        assert_eq!(
            expanded,
            vec!["hello", "tried", "try", "rework", "reworked", "work", "worked",]
        )
    }

    #[test]
    fn plural_giants() {
        let words = parse_word_list("1\ngiant/SM").unwrap();
        let attributes = AttributeList::parse(
            "SFX S Y 4\nSFX S   y     ies        [^aeiou]y\nSFX S   0     s          \
             [aeiou]y\nSFX S   0     es         [sxzh]\nSFX S   0     s          [^sxzhy]\n\nSFX \
             M Y 1\nSFX M   0     's         .",
        )
        .unwrap();

        let mut expanded = Vec::new();

        attributes.expand_marked_words(words, &mut expanded);

        assert!(expanded.contains(&split("giants")))
    }

    fn build_expanded() -> Vec<CharString> {
        let words = parse_default_word_list().unwrap();
        let attributes = parse_default_attribute_list().unwrap();

        let mut expanded = Vec::new();

        attributes.expand_marked_words(words, &mut expanded);

        expanded
    }

    #[test]
    fn can_expand_default() {
        build_expanded();
    }

    #[test]
    fn expanded_contains_giants() {
        assert!(build_expanded().contains(&split("giants")));
    }

    #[test]
    fn expanded_contains_deallocate() {
        assert!(build_expanded().contains(&split("deallocate")));
    }

    fn split(text: &str) -> CharString {
        text.chars().collect()
    }
}

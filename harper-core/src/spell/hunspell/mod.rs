mod attributes;
mod error;
mod matcher;
mod word_list;

use self::word_list::parse_word_list;
pub use self::word_list::MarkedWord;
pub use attributes::AttributeList;
pub use error::Error;

pub fn parse_default_word_list() -> Result<Vec<MarkedWord>, Error> {
    parse_word_list(include_str!("../../../../dictionary.dict"))
}

pub fn parse_default_attribute_list() -> Result<AttributeList, Error> {
    AttributeList::parse(include_str!("../../../../dictionary.aff"))
}

#[cfg(test)]
mod tests {
    use super::{
        attributes::AttributeList, parse_default_attribute_list, parse_default_word_list,
        word_list::parse_word_list,
    };

    pub const TEST_WORD_LIST: &str = "3\nhello\ntry/B\nwork/AB";
    pub const ATTR_LIST: &str = "SET UTF-8\nTRY esianrtolcdugmphbyfvkwzESIANRTOLCDUGMPHBYFVKWZ'\n\nREP 2\nREP f ph\nREP ph f\n\nPFX A Y 1\nPFX A 0 re .\n\nSFX B Y 2\nSFX B 0 ed [^y]\nSFX B y ied y";

    #[test]
    fn correctly_expands_test_files() {
        let words = parse_word_list(TEST_WORD_LIST).unwrap();
        let attributes = AttributeList::parse(ATTR_LIST).unwrap();

        dbg!(&attributes);

        let expanded = attributes.expand_marked_words(words).unwrap();
        let expanded: Vec<String> = expanded
            .into_iter()
            .map(|v| v.into_iter().collect())
            .collect();

        assert_eq!(
            expanded,
            vec!["hello", "tried", "try", "reworked", "rework", "worked", "work"]
        )
    }

    #[test]
    fn plural_giants() {
        let words = parse_word_list("1\ngiant/SM").unwrap();
        let attributes = AttributeList::parse(
            "SFX S Y 4\nSFX S   y     ies        [^aeiou]y\nSFX S   0     s          [aeiou]y\nSFX S   0     es         [sxzh]\nSFX S   0     s          [^sxzhy]\n\nSFX M Y 1\nSFX M   0     's         .",
        )
        .unwrap();

        let expanded = attributes.expand_marked_words(words).unwrap();
        assert!(expanded.contains(&split("giants")))
    }

    fn build_expanded() -> Vec<Vec<char>> {
        let words = parse_default_word_list().unwrap();
        let attributes = parse_default_attribute_list().unwrap();

        dbg!(&attributes);

        attributes.expand_marked_words(words).unwrap()
    }

    #[test]
    fn can_expand_default() {
        build_expanded();
    }

    #[test]
    fn expanded_contains_giants() {
        assert!(build_expanded().contains(&split("giants")));
    }

    fn split(text: &str) -> Vec<char> {
        text.chars().collect()
    }
}

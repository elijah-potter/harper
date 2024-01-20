mod attributes;
mod error;
mod matcher;
mod word_list;

pub use error::Error;

#[cfg(test)]
mod tests {
    use super::{attributes::AttributeList, word_list::parse_word_list};

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
}

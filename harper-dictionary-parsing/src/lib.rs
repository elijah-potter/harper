mod affix_replacement;
mod attribute_list;
mod error;
mod expansion;
mod matcher;
mod word_list;

pub use attribute_list::AttributeList;
use attribute_list::HumanReadableAttributeList;
pub use error::Error;
pub use word_list::{parse_word_list, MarkedWord};

pub fn parse_default_word_list() -> Result<Vec<MarkedWord>, Error> {
    parse_word_list(include_str!("../dictionary.dict"))
}

pub fn parse_default_attribute_list() -> AttributeList {
    let human_readable: HumanReadableAttributeList =
        serde_json::from_str(include_str!("../affixes.json"))
            .expect("The built-in affix list should always be valid.");

    human_readable
        .into_normal()
        .expect("All expressions in the built-in attribute list should be valid.")
}

#[cfg(test)]
mod tests {
    use harper_data::{CharString, WordMetadata};
    use hashbrown::{HashMap, HashSet};
    use serde_json::json;

    use super::word_list::parse_word_list;
    use super::{parse_default_attribute_list, parse_default_word_list};
    use crate::attribute_list::HumanReadableAttributeList;

    pub const TEST_WORD_LIST: &str = "3\nhello\ntry/B\nwork/AB";

    #[test]
    fn correctly_expands_test_files() {
        let words = parse_word_list(TEST_WORD_LIST).unwrap();
        let attributes: HumanReadableAttributeList = serde_json::from_value(json!({
            "affixes": {
                "A": {
                    "suffix": false,
                    "cross_product": true,
                    "replacements": [
                      {
                        "remove": "",
                        "add": "re",
                        "condition": "."
                      }
                    ],
                    "adds_metadata": {
                      "kind": null,
                      "tense": null
                    },
                    "gifts_metadata": {}
                },
                "B": {
                    "suffix": true,
                    "cross_product": true,
                    "replacements": [
                      {
                        "remove": "",
                        "add": "ed",
                        "condition": "[^y]"
                      },
                      {
                        "remove": "y",
                        "add": "ied",
                        "condition": "y"
                      }
                    ],
                    "adds_metadata": {
                      "kind": null,
                      "tense": null
                    },
                    "gifts_metadata": {}
                }
            }
        }))
        .unwrap();
        let attributes = attributes.into_normal().unwrap();

        let mut expanded = HashMap::new();

        attributes.expand_marked_words(words, &mut expanded);
        let expanded: HashSet<String> = expanded
            .into_iter()
            .map(|v| v.0.into_iter().collect())
            .collect();

        assert_eq!(
            expanded,
            vec!["hello", "tried", "reworked", "rework", "worked", "work", "try"]
                .into_iter()
                .map(|v| v.into())
                .collect()
        )
    }

    #[test]
    fn plural_giants() {
        let words = parse_word_list("1\ngiant/SM").unwrap();

        let attributes: HumanReadableAttributeList = serde_json::from_value(json!({
            "affixes": {
                "S": {
                    "suffix": true,
                    "cross_product": true,
                    "replacements": [
                      {
                        "remove": "y",
                        "add": "ies",
                        "condition": "[^aeiou]"
                      },
                      {
                        "remove": "",
                        "add": "s",
                        "condition": "[aeiou]y"
                      },
                      {
                        "remove": "",
                        "add": "s",
                        "condition": "[^sxzhy]"
                      }
                    ],
                    "adds_metadata": {
                        "noun": {
                            "is_plural": true
                        }
                    },
                    "gifts_metadata": {
                        "noun": {}
                    }
                },
                "M": {
                    "suffix": true,
                    "cross_product": true,
                    "replacements": [
                      {
                        "remove": "",
                        "add": "'s",
                        "condition": "."
                      }
                    ],
                    "adds_metadata": {},
                    "gifts_metadata": {}
                }
            }
        }))
        .unwrap();
        let attributes = attributes.into_normal().unwrap();

        let mut expanded: HashMap<CharString, WordMetadata> = HashMap::new();

        attributes.expand_marked_words(words, &mut expanded);

        let giant_data = expanded.get(&split("giant")).unwrap();
        assert!(giant_data.is_noun());

        let giants_data = expanded.get(&split("giants")).unwrap();
        assert!(giants_data.is_plural_noun());
    }

    fn build_expanded() -> HashMap<CharString, WordMetadata> {
        let words = parse_default_word_list().unwrap();
        let attributes = parse_default_attribute_list();

        let mut expanded = HashMap::new();

        attributes.expand_marked_words(words, &mut expanded);

        expanded
    }

    #[test]
    fn can_expand_default() {
        build_expanded();
    }

    #[test]
    fn expanded_contains_giants() {
        assert!(build_expanded().contains_key(&split("giants")));
    }

    #[test]
    fn expanded_contains_deallocate() {
        assert!(build_expanded().contains_key(&split("deallocate")));
    }

    #[test]
    fn expanded_contains_repo() {
        let expanded = build_expanded();

        assert!(expanded.contains_key(&split("repo")));
        assert!(expanded.contains_key(&split("repos")));
        assert!(expanded.contains_key(&split("repo's")));
    }

    #[test]
    fn expanded_contains_possessive_abandonment() {
        assert!(build_expanded()
            .get(&split("abandonment's"))
            .unwrap()
            .is_possessive_noun())
    }

    #[test]
    fn has_is_not_a_noun() {
        let expanded = build_expanded();

        let has = expanded.get(&split("has"));
        assert!(has.is_some());

        assert!(!has.unwrap().is_noun(),)
    }

    #[test]
    fn is_is_linking_verb() {
        let expanded = build_expanded();

        let is = expanded.get(&split("is"));

        dbg!(&is);
        assert!(is.is_some());
        assert!(is.unwrap().is_linking_verb());
    }

    fn split(text: &str) -> CharString {
        text.chars().collect()
    }
}

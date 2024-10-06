use crate::Lrc;
use std::collections::VecDeque;

use itertools::Itertools;

use super::{Parser, TokenKind};
use crate::patterns::{PatternExt, SequencePattern};
use crate::{Dictionary, FullDictionary, MergedDictionary, Span, Token, VecExt};

/// A parser that wraps any other parser to collapse token strings that match
/// the pattern `word_word` or `word-word`.
pub struct CollapseIdentifiers {
    inner: Box<dyn Parser>,
    dict: Lrc<MergedDictionary<FullDictionary>>,
}

impl CollapseIdentifiers {
    pub fn new(inner: Box<dyn Parser>, dict: &Lrc<MergedDictionary<FullDictionary>>) -> Self {
        Self {
            inner,
            dict: dict.clone(),
        }
    }
}

thread_local! {
    static WORD_OR_NUMBER: Lrc<SequencePattern> = Lrc::new(SequencePattern::default()
                .then_any_word()
                .then_one_or_more(Box::new(SequencePattern::default()
        .then_case_separator()
        .then_any_word())));
}

impl Parser for CollapseIdentifiers {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut tokens = self.inner.parse(source);

        let mut to_remove = VecDeque::default();

        for tok_span in WORD_OR_NUMBER
            .with(|v| v.clone())
            .find_all_matches(&tokens, source)
        {
            let start_tok = &tokens[tok_span.start];
            let end_tok = &tokens[tok_span.end - 1];
            let char_span = Span::new(start_tok.span.start, end_tok.span.end);

            if self.dict.contains_word(char_span.get_content(source)) {
                tokens[tok_span.start] = Token::new(char_span, TokenKind::blank_word());
                to_remove.extend(tok_span.start + 1..tok_span.end);
            }
        }

        tokens.remove_indices(to_remove.into_iter().sorted().unique().collect());

        tokens
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parsers::{PlainEnglish, StrParser},
        WordMetadata,
    };

    use super::*;

    #[test]
    fn matches_kebab() {
        let source: Vec<_> = "kebab-case".chars().collect();

        assert_eq!(
            WORD_OR_NUMBER
                .with(|v| v.clone())
                .find_all_matches(&PlainEnglish.parse(&source), &source)
                .len(),
            1
        );
    }

    #[test]
    fn no_collapse() {
        let dict = FullDictionary::curated();
        let source = "This is a test.";

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish), &Lrc::new(dict.into()))
            .parse_str(source);
        assert_eq!(tokens.len(), 8);
    }

    #[test]
    fn one_collapse() {
        let source = "This is a separated_identifier, wow!";
        let default_dict = FullDictionary::curated();

        let tokens = CollapseIdentifiers::new(
            Box::new(PlainEnglish),
            &Lrc::new(default_dict.clone().into()),
        )
        .parse_str(source);
        assert_eq!(tokens.len(), 13);

        let mut dict = FullDictionary::new();
        dict.append_word_str("separated_identifier", WordMetadata::default());

        let mut merged_dict = MergedDictionary::from(default_dict);
        merged_dict.add_dictionary(Lrc::new(dict));

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish), &Lrc::new(merged_dict))
            .parse_str(source);
        assert_eq!(tokens.len(), 11);
    }

    #[test]
    fn kebab_collapse() {
        let source = "This is a separated-identifier, wow!";
        let default_dict = FullDictionary::curated();

        let tokens = CollapseIdentifiers::new(
            Box::new(PlainEnglish),
            &Lrc::new(default_dict.clone().into()),
        )
        .parse_str(source);

        assert_eq!(tokens.len(), 13);

        let mut dict = FullDictionary::new();
        dict.append_word_str("separated-identifier", WordMetadata::default());

        let mut merged_dict = MergedDictionary::from(default_dict);
        merged_dict.add_dictionary(Lrc::new(dict));

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish), &Lrc::new(merged_dict))
            .parse_str(source);

        assert_eq!(tokens.len(), 11);
    }

    #[test]
    fn double_collapse() {
        let source = "This is a separated_identifier_token, wow!";
        let default_dict = FullDictionary::curated();

        let tokens = CollapseIdentifiers::new(
            Box::new(PlainEnglish),
            &Lrc::new(default_dict.clone().into()),
        )
        .parse_str(source);
        assert_eq!(tokens.len(), 15);

        let mut dict = FullDictionary::new();
        dict.append_word_str("separated_identifier_token", WordMetadata::default());

        let mut merged_dict = MergedDictionary::from(default_dict);
        merged_dict.add_dictionary(Lrc::new(dict));

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish), &Lrc::new(merged_dict))
            .parse_str(source);
        assert_eq!(tokens.len(), 11);
    }

    #[test]
    fn two_collapses() {
        let source = "This is a separated_identifier, wow! separated_identifier";
        let default_dict = FullDictionary::curated();

        let tokens = CollapseIdentifiers::new(
            Box::new(PlainEnglish),
            &Lrc::new(default_dict.clone().into()),
        )
        .parse_str(source);
        assert_eq!(tokens.len(), 17);

        let mut dict = FullDictionary::new();
        dict.append_word_str("separated_identifier", WordMetadata::default());

        let mut merged_dict = MergedDictionary::from(default_dict);
        merged_dict.add_dictionary(Lrc::new(dict));

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish), &Lrc::new(merged_dict))
            .parse_str(source);
        assert_eq!(tokens.len(), 13);
    }

    #[test]
    fn overlapping_identifiers() {
        let source = "This is a separated_identifier_token, wow!";
        let default_dict = FullDictionary::curated();

        let tokens = CollapseIdentifiers::new(
            Box::new(PlainEnglish),
            &Lrc::new(default_dict.clone().into()),
        )
        .parse_str(source);
        assert_eq!(tokens.len(), 15);

        let mut dict = FullDictionary::new();
        dict.append_word_str("separated_identifier", WordMetadata::default());
        dict.append_word_str("identifier_token", WordMetadata::default());

        let mut merged_dict = MergedDictionary::from(default_dict);
        merged_dict.add_dictionary(Lrc::new(dict));

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish), &Lrc::new(merged_dict))
            .parse_str(source);
        assert_eq!(tokens.len(), 15);
    }

    #[test]
    fn nested_identifiers() {
        let source = "This is a separated_identifier_token, wow!";
        let default_dict = FullDictionary::curated();

        let tokens = CollapseIdentifiers::new(
            Box::new(PlainEnglish),
            &Lrc::new(default_dict.clone().into()),
        )
        .parse_str(source);
        assert_eq!(tokens.len(), 15);

        let mut dict = FullDictionary::new();
        dict.append_word_str("separated_identifier_token", WordMetadata::default());
        dict.append_word_str("separated_identifier", WordMetadata::default());

        let mut merged_dict = MergedDictionary::from(default_dict);
        merged_dict.add_dictionary(Lrc::new(dict));

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish), &Lrc::new(merged_dict))
            .parse_str(source);
        assert_eq!(tokens.len(), 11);
    }
}

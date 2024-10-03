use std::collections::VecDeque;
use std::rc::Rc as Lrc;
use std::sync::Arc;

use itertools::Itertools;

use super::{Parser, TokenKind};
use crate::patterns::{PatternExt, SequencePattern};
use crate::{Dictionary, FullDictionary, MergedDictionary, Span, Token, VecExt, WordMetadata};

/// A parser that wraps any other parser to collapse token strings that match
/// the pattern word_word or word-word.
pub struct Cases {
    inner: Box<dyn Parser>,
    dict: Arc<MergedDictionary<FullDictionary>>,
}

impl Cases {
    pub fn new(inner: Box<dyn Parser>, dict: &Arc<MergedDictionary<FullDictionary>>) -> Self {
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

impl Parser for Cases {
    /// This implementation is quite gross to look at, but it works.
    /// If any issues arise, it would likely help to refactor this out first.
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut tokens = self.inner.parse(source);

        // TODO: change is_case_separator to is_underscore and is_hyphen
        // TODO: Make Sequence Patterns thread_local statics

        let mut removal_indexes: VecDeque<usize> = VecDeque::default();
        let replacements = WORD_OR_NUMBER
            .with(|v| v.clone())
            .find_all_matches(&tokens, source)
            .into_iter()
            .map(|s| {
                let start_tok = tokens
                    .get(s.start)
                    .expect("Token not at expected position.");
                let end_tok = tokens
                    .get(s.end - 1)
                    .expect("Token not at expected position.");
                let char_span = Span::new(start_tok.span.start, end_tok.span.end);
                (
                    s.start,
                    s.end,
                    Token::new(char_span, TokenKind::Word(WordMetadata::default())),
                    char_span.get_content_string(source),
                )
            })
            .filter(|(_, _, _, st)| self.dict.contains_word_str(st))
            .collect_vec();

        // The indexes will be shifted after each removal, so this won't work
        replacements.into_iter().for_each(|(s, e, t, _)| {
            (s + 1..=e).for_each(|n| removal_indexes.push_front(n));
            tokens[s] = t;
        });
        tokens.remove_indices(removal_indexes.into_iter().sorted().unique().collect());

        tokens
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::{PlainEnglish, StrParser};

    use super::*;

    #[test]
    fn no_collapse() {
        let mut dict = FullDictionary::curated();
        let source = "This is a test.";

        let tokens = Cases::new(Box::new(PlainEnglish), &Arc::new(dict.into())).parse_str(source);
        assert_eq!(tokens.len(), 8);
    }

    #[test]
    fn one_collapse() {
        let source = "This is a separated_identifier, wow!";
        let mut dict = FullDictionary::curated();

        let tokens =
            Cases::new(Box::new(PlainEnglish), &Arc::new(dict.clone().into())).parse_str(source);
        assert_eq!(tokens.len(), 13);

        dict.append_word(
            "separated_identifier".chars().collect_vec(),
            WordMetadata::default(),
        );

        let tokens = Cases::new(Box::new(PlainEnglish), &Arc::new(dict.into())).parse_str(source);
        assert_eq!(tokens.len(), 10);
    }

    #[test]
    fn double_collapse() {
        let source = "This is a separated_identifier_token, wow!";
        let mut dict = FullDictionary::curated();

        let tokens = Cases::new(Box::new(PlainEnglish), &Arc::new(dict.into())).parse_str(source);
        assert_eq!(tokens.len(), 15);

        let mut new_dict = FullDictionary::new();
        new_dict.append_word(
            "separated_identifier".chars().collect_vec(),
            WordMetadata::default(),
        );
        new_dict.append_word(
            "separated_identifier_token".chars().collect_vec(),
            WordMetadata::default(),
        );
        dict.add_dictionary(Arc::new(new_dict));

        let tokens = Cases::new(Box::new(PlainEnglish), &dict).parse_str(source);
        assert_eq!(tokens.len(), 10);
    }

    #[test]
    fn two_collapses() {
        let source = "This is a separated_identifier, wow! separated_identifier";
        let mut dict = FullDictionary::curated();

        let tokens = Cases::new(Box::new(PlainEnglish), &dict).parse_str(source);
        assert_eq!(tokens.len(), 17);

        dict.append_word(
            "separated_identifier".chars().collect_vec(),
            WordMetadata::default(),
        );

        let tokens = Cases::new(Box::new(PlainEnglish), &dict).parse_str(source);
        assert_eq!(tokens.len(), 12);
    }
}

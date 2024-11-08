use std::collections::VecDeque;

use harper_data::{Lrc, Span, Token, TokenKind, VecExt};
use itertools::Itertools;

use super::Parser;
use harper_patterns::{PatternExt, SequencePattern};

/// A parser that wraps any other parser to collapse token strings that match
/// the pattern `word_word` or `word-word`.
pub struct CollapseIdentifiers {
    inner: Box<dyn Parser>,
}

impl CollapseIdentifiers {
    pub fn new(inner: Box<dyn Parser>) -> Self {
        Self { inner }
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

            tokens[tok_span.start] = Token::new(char_span, TokenKind::blank_word());
            to_remove.extend(tok_span.start + 1..tok_span.end);
        }

        tokens.remove_indices(to_remove.into_iter().sorted().unique().collect());

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harper_parsing::{PlainEnglish, StrParser};

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
        let source = "This is a test.";

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish)).parse_str(source);
        assert_eq!(tokens.len(), 8);
    }

    #[test]
    fn one_collapse() {
        let source = "This is a separated_identifier, wow!";

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish)).parse_str(source);
        assert_eq!(tokens.len(), 11);
    }

    #[test]
    fn kebab_collapse() {
        let source = "This is a separated-identifier, wow!";

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish)).parse_str(source);

        assert_eq!(tokens.len(), 11);
    }

    #[test]
    fn double_collapse() {
        let source = "This is a separated_identifier_token, wow!";

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish)).parse_str(source);
        assert_eq!(tokens.len(), 11);
    }

    #[test]
    fn two_collapses() {
        let source = "This is a separated_identifier, wow! separated_identifier";

        let tokens = CollapseIdentifiers::new(Box::new(PlainEnglish)).parse_str(source);
        assert_eq!(tokens.len(), 13);
    }
}

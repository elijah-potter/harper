use harper_core::parsers::{Markdown, Parser};
use harper_core::Token;

use super::without_initiators;

/// A comment parser that strips starting `/` and `*` characters.
///
/// It is meant to cover _most_ cases in _most_ programming languages.
///
/// It assumes it is being provided a single line of comment at a time,
/// including the comment initiation characters.
pub struct Unit;

impl Parser for Unit {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let actual = without_initiators(source);

        if actual.is_empty() {
            return Vec::new();
        }

        let source = actual.get_content(source);

        let mut markdown_parser = Markdown;

        let mut new_tokens = markdown_parser.parse(source);

        new_tokens
            .iter_mut()
            .for_each(|t| t.span.push_by(actual.start));

        new_tokens
    }
}

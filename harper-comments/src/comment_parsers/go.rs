use harper_core::parsers::{Markdown, Parser};
use harper_core::Token;

use super::without_initiators;

#[derive(Clone)]
pub struct Go {
    markdown_parser: Markdown,
}

impl Go {
    pub fn new(markdown_parser: Markdown) -> Self {
        Self { markdown_parser }
    }
}

impl Parser for Go {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut actual = without_initiators(source);
        let mut actual_source = actual.get_content(source);

        if matches!(actual_source, ['g', 'o', ':', ..]) {
            let Some(terminator) = source.iter().position(|c| *c == '\n') else {
                return Vec::new();
            };

            actual.start += terminator;

            let Some(new_source) = actual.try_get_content(actual_source) else {
                return Vec::new();
            };

            actual_source = new_source
        }

        let mut new_tokens = self.markdown_parser.parse(actual_source);

        new_tokens
            .iter_mut()
            .for_each(|t| t.span.push_by(actual.start));

        new_tokens
    }
}

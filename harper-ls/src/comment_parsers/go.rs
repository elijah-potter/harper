use harper_core::parsers::{Markdown, Parser};
use harper_core::Token;

use super::without_intiators;

#[derive(Debug, Clone, Copy)]
pub struct Go;

impl Parser for Go {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut actual = without_intiators(source);
        let mut actual_source = actual.get_content(source);

        if matches!(source, ['g', 'o', ':', ..]) {
            let Some(terminator) = source.iter().position(|c| c.is_whitespace()) else {
                return Vec::new();
            };

            actual.start += terminator;

            let Some(new_source) = actual.try_get_content(actual_source) else {
                return Vec::new();
            };

            actual_source = new_source
        }

        let mut markdown_parser = Markdown;

        let mut new_tokens = markdown_parser.parse(actual_source);

        new_tokens
            .iter_mut()
            .for_each(|t| t.span.offset(actual.start));

        new_tokens
    }
}

#[cfg(test)]
mod tests {}

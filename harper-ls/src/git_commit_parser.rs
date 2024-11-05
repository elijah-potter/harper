use harper_core::parsers::{Markdown, Parser};
use harper_data::Token;

/// A Harper parser for Git commit files
pub struct GitCommitParser;

impl Parser for GitCommitParser {
    /// Admittedly a somewhat naive implementation.
    /// We're going to get _something_ to work, before we polish it off.
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        // Locate the first `#`
        let end = source
            .iter()
            .position(|c| *c == '#')
            .unwrap_or(source.len());

        let mut md_parser = Markdown;

        md_parser.parse(&source[0..end])
    }
}

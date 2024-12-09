use harper_core::parsers::{Markdown, Parser};

/// A Harper parser for Git commit files
#[derive(Clone)]
pub struct GitCommitParser {
    markdown_parser: Markdown,
}

impl GitCommitParser {
    pub fn new(markdown_parser: Markdown) -> Self {
        Self { markdown_parser }
    }
}

impl Parser for GitCommitParser {
    /// Admittedly a somewhat naive implementation.
    /// We're going to get _something_ to work, before we polish it off.
    fn parse(&mut self, source: &[char]) -> Vec<harper_core::Token> {
        // Locate the first `#`
        let end = source
            .iter()
            .position(|c| *c == '#')
            .unwrap_or(source.len());

        self.markdown_parser.parse(&source[0..end])
    }
}

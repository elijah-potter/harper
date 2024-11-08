use harper_data::Token;
use harper_parsing::{Markdown, Parser};

/// A Harper parser for Git commit files.
///
/// In this crate since the only place it's needed at the moment is the Gestalt parser.
/// If it needs to be used _without_ the rest of the Gestalt parser, feel free to move it to it's
/// own crate.
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

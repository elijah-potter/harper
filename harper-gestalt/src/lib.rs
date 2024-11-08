mod collapse_identifiers;
mod git_commit_parser;

use std::path::Path;

use collapse_identifiers::CollapseIdentifiers;
use git_commit_parser::GitCommitParser;
use harper_comments::CommentParser;
use harper_data::Token;
use harper_html::HtmlParser;
use harper_parsing::{Markdown, Parser, PlainEnglish};

/// A [`Parser`](harper_parsing::Parser) that combines a variety of parsers to
/// singlehandedly support a significant variety of programming languages and
/// file formats.
///
/// For now, it just allows us to provide a filetype and get a parser.
/// Eventually, we plan to support nesting (like linting the comments inside
/// Markdown code blocks).
pub struct GestaltParser {
    inner: Box<dyn Parser>,
}

impl GestaltParser {
    pub fn new_from_language_id(language_id: &str) -> Option<Self> {
        let inner: Box<dyn Parser> =
            if let Some(ts_parser) = CommentParser::new_from_language_id(language_id) {
                Box::new(CollapseIdentifiers::new(Box::new(ts_parser)))
            } else if language_id == "markdown" {
                Box::new(Markdown)
            } else if language_id == "gitcommit" {
                Box::new(GitCommitParser)
            } else if language_id == "html" {
                Box::new(HtmlParser::default())
            } else if language_id == "mail" {
                Box::new(PlainEnglish)
            } else {
                return None;
            };

        Some(Self { inner })
    }

    /// Infer the programming language from a provided filename.
    pub fn new_from_filename(filename: &Path) -> Option<Self> {
        let inner = match Self::filename_to_filetype(filename) {
            Some(filetype) => return Self::new_from_language_id(filetype),
            None => Box::new(CommentParser::new_from_filename(filename)?),
        };

        Some(Self { inner })
    }

    /// Convert a provided path to a corresponding Language Server Protocol file
    /// type.
    ///
    /// Note to contributors: try to keep this in sync with
    /// [`Self::new_from_language_id`].
    ///
    /// This operates in _addition_ to the similarly named function in the
    /// [`CommentParser`].
    fn filename_to_filetype(path: &Path) -> Option<&'static str> {
        Some(match path.extension()?.to_str()? {
            "md" => "markdown",
            "html" => "html",
            _ => return None,
        })
    }
}

impl Parser for GestaltParser {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}

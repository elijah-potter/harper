use harper_core::{
    parsers::{Markdown, Parser},
    Span,
};
use tree_sitter::{Language, TreeCursor};

/// A Harper parser that wraps the standard [`Markdown`] parser that exclusively parses
/// comments in any language supported by [`tree_sitter`].
pub struct TreeSitterParser {
    language: Language,
}

impl TreeSitterParser {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    pub fn new_from_extension(file_extension: &str) -> Option<Self> {
        let language = match file_extension {
            "rs" => tree_sitter_rust::language(),
            "tsx" => tree_sitter_typescript::language_tsx(),
            "ts" => tree_sitter_typescript::language_typescript(),
            "py" => tree_sitter_python::language(),
            "js" => tree_sitter_javascript::language(),
            "go" => tree_sitter_go::language(),
            "c" => tree_sitter_c::language(),
            "cpp" => tree_sitter_cpp::language(),
            "h" => tree_sitter_cpp::language(),
            "hpp" => tree_sitter_cpp::language(),
            "rb" => tree_sitter_ruby::language(),
            "swift" => tree_sitter_ruby::language(),
            "cs" => tree_sitter_c_sharp::language(),
            "toml" => tree_sitter_toml::language(),
            "lua" => tree_sitter_lua::language(),
            _ => return None,
        };

        Some(Self { language })
    }
}

impl Parser for TreeSitterParser {
    fn parse(&mut self, source: &[char]) -> Vec<harper_core::Token> {
        let text: String = source.iter().collect();

        let mut markdown_parser = Markdown;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(self.language).unwrap();

        // TODO: Use incremental parsing
        let Some(root) = parser.parse(&text, None) else {
            return vec![];
        };

        let mut comments_spans = Vec::new();

        extract_comments(&mut root.walk(), &mut comments_spans);
        byte_spans_to_char_spans(&mut comments_spans, &text);

        let mut tokens = Vec::new();

        for span in comments_spans {
            // Skip over the comment start characters
            let actual_start = source[span.start..span.end]
                .iter()
                .position(|c| !is_comment_character(*c))
                .unwrap_or(0)
                + span.start;

            if span.end <= actual_start {
                continue;
            }

            let mut new_tokens = markdown_parser.parse(&source[actual_start..span.end]);

            new_tokens
                .iter_mut()
                .for_each(|t| t.span.offset(actual_start));

            tokens.append(&mut new_tokens);
        }

        tokens
    }
}

fn is_comment_character(c: char) -> bool {
    matches!(c, '#' | '-' | '/')
}

/// Converts a set of byte-indexed [`Span`]s to char-index Spans, in-place.
/// NOTE: Will sort the given array by their [`Span::start`].
///
/// Assumes that none of the Spans are overlapping.
fn byte_spans_to_char_spans(byte_spans: &mut [Span], source: &str) {
    byte_spans.sort_by_key(|s| s.start);

    let mut last_byte_pos = 0;
    let mut last_char_pos = 0;

    byte_spans.iter_mut().for_each(|span| {
        let byte_span = *span;

        last_char_pos += source[last_byte_pos..byte_span.start].chars().count();
        span.start = last_char_pos;

        last_char_pos += source[byte_span.start..byte_span.end].chars().count();
        span.end = last_char_pos;

        last_byte_pos = byte_span.end;
    })
}

/// Visits the children of a TreeSitter node, searching for comments.
///
/// Returns the BYTE spans of the comment position.
fn extract_comments(cursor: &mut TreeCursor, comments: &mut Vec<Span>) {
    if !cursor.goto_first_child() {
        return;
    }

    while cursor.goto_next_sibling() {
        let node = cursor.node();

        if node.kind().contains("comment") {
            comments.push(node.byte_range().into());
        }

        extract_comments(cursor, comments);
    }

    cursor.goto_parent();
}

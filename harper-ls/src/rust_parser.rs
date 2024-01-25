use harper_core::{
    parsers::{Parser, PlainEnglishParser},
    Span,
};
use tree_sitter::TreeCursor;

pub struct RustParser;

impl Parser for RustParser {
    fn parse(&mut self, source: &[char]) -> Vec<harper_core::Token> {
        let text: String = source.iter().collect();

        let mut english_parser = PlainEnglishParser;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();

        // TODO: Use incremental parsing
        let Some(root) = parser.parse(&text, None) else {
            return vec![];
        };

        let mut comments_spans = Vec::new();

        extract_comments(&mut root.walk(), &mut comments_spans);
        byte_spans_to_char_spans(&mut comments_spans, &text);

        let mut tokens = Vec::new();

        for span in comments_spans {
            let mut new_tokens = english_parser.parse(&source[span.start..span.end]);

            new_tokens
                .iter_mut()
                .for_each(|t| t.span.offset(span.start));

            tokens.append(&mut new_tokens);
        }

        tokens
    }
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

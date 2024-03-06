use std::collections::HashSet;

use harper_core::parsers::Parser;
use harper_core::{FullDictionary, Span, Token};
use tree_sitter::{Language, Node, Tree, TreeCursor};

use super::comment_parsers::{Go, Unit};

/// A Harper parser that wraps various [`super::comment_parsers`] that
/// exclusively parses comments in any language supported by [`tree_sitter`].
pub struct TreeSitterParser {
    language: Language,
    comment_parser: Box<dyn Parser>
}

impl TreeSitterParser {
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
            "swift" => tree_sitter_swift::language(),
            "cs" => tree_sitter_c_sharp::language(),
            "toml" => tree_sitter_toml::language(),
            "lua" => tree_sitter_lua::language(),
            _ => return None
        };

        let comment_parser: Box<dyn Parser> = match file_extension {
            "go" => Box::new(Go),
            _ => Box::new(Unit)
        };

        Some(Self {
            language,
            comment_parser
        })
    }

    fn parse_root(&self, text: &str) -> Option<Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(self.language).unwrap();

        // TODO: Use incremental parsing
        parser.parse(text, None)
    }

    pub fn create_ident_dict(&self, source: &[char]) -> Option<FullDictionary> {
        let text: String = source.iter().collect();

        // Byte-indexed
        let mut ident_spans = Vec::new();

        let tree = self.parse_root(&text)?;
        Self::visit_nodes(&mut tree.walk(), &mut |node: &Node| {
            if node.child_count() == 0 && node.kind().contains("ident") {
                ident_spans.push(node.byte_range().into())
            }
        });

        byte_spans_to_char_spans(&mut ident_spans, &text);

        let mut idents = HashSet::new();

        for span in ident_spans {
            idents.insert(span.get_content(source));
        }

        let idents: Vec<_> = idents.into_iter().collect();

        let mut dictionary = FullDictionary::new();
        dictionary.extend_words(idents);

        Some(dictionary)
    }

    /// Visits the children of a TreeSitter node, searching for comments.
    ///
    /// Returns the BYTE spans of the comment position.
    fn extract_comments(cursor: &mut TreeCursor, comments: &mut Vec<Span>) {
        Self::visit_nodes(cursor, &mut |node: &Node| {
            if node.kind().contains("comment") {
                comments.push(node.byte_range().into());
            }
        });
    }

    fn visit_nodes(cursor: &mut TreeCursor, visit: &mut impl FnMut(&Node)) {
        if !cursor.goto_first_child() {
            return;
        }

        loop {
            let node = cursor.node();

            visit(&node);

            Self::visit_nodes(cursor, visit);

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
    }
}

impl Parser for TreeSitterParser {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let text: String = source.iter().collect();

        let Some(root) = self.parse_root(&text) else {
            return vec![];
        };

        let mut comments_spans = Vec::new();

        Self::extract_comments(&mut root.walk(), &mut comments_spans);

        dbg!(&comments_spans.len());
        byte_spans_to_char_spans(&mut comments_spans, &text);
        dbg!(&comments_spans.len());

        let mut tokens = Vec::new();

        for (s_index, span) in comments_spans.iter().enumerate() {
            let mut new_tokens = self.comment_parser.parse(span.get_content(source));

            new_tokens
                .iter_mut()
                .for_each(|v| v.span.offset(span.start));

            // The comment parser will insert a newline at end-of-input.
            // If the next treesitter chunk is a comment, we want to remove that.
            if let Some(next_start) = comments_spans.get(s_index + 1).map(|v| v.start) {
                if is_span_whitespace(Span::new(span.end, next_start), source) {
                    new_tokens.pop();
                }
            }

            tokens.append(&mut new_tokens);
        }

        tokens
    }
}

/// Check if the contents of a span is just white-space.
fn is_span_whitespace(span: Span, source: &[char]) -> bool {
    span.get_content(source)
        .iter()
        .filter(|c| !c.is_whitespace())
        .count()
        == 0
}

/// Converts a set of byte-indexed [`Span`]s to char-index Spans, in-place.
/// NOTE: Will sort the given slice by their [`Span::start`].
///
/// If any spans overlap, it will remove the second one.
fn byte_spans_to_char_spans(byte_spans: &mut Vec<Span>, source: &str) {
    byte_spans.sort_by_key(|s| s.start);

    let cloned = byte_spans.clone();

    let mut i: usize = 0;
    byte_spans.retain(|cur| {
        i += 1;
        if let Some(prev) = cloned.get(i.wrapping_sub(2)) {
            !cur.overlaps_with(*prev)
        } else {
            true
        }
    });

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

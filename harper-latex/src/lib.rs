use harper_core::parsers::{self, Parser, PlainEnglish};
use harper_core::Span;
use harper_core::Token;
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct LatexParser {
    inner: parsers::Mask<TreeSitterMasker, PlainEnglish>,
}

impl LatexParser {
    fn node_condition(n: &Node) -> bool {
        fn ancestor_contains(node: &Node, s: &str) -> bool {
            if let Some(parent) = &node.parent() {
                if parent.kind().contains(s) {
                    true
                } else {
                    ancestor_contains(parent, s)
                }
            } else {
                false
            }
        }

        let mut cursor = n.walk();
        n.kind() == "word"
            && n.children(&mut cursor)
                .into_iter()
                .all(|c| c.kind().contains("command"))
            && !ancestor_contains(n, "include")
    }
}

impl Default for LatexParser {
    fn default() -> Self {
        Self {
            inner: parsers::Mask::new(
                TreeSitterMasker::new(tree_sitter_latex::language(), Self::node_condition),
                PlainEnglish,
            ),
        }
    }
}

impl Parser for LatexParser {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut tokens = Vec::new();

        let mut chars_traversed = 0;

        for line in source.split(|c| *c == '\n') {
            let mut new_tokens = parse_line(line, &mut self.inner);

            new_tokens.push(Token::new(
                Span::new_with_len(line.len(), 1),
                harper_core::TokenKind::Newline(1),
            ));

            new_tokens
                .iter_mut()
                .for_each(|t| t.span.push_by(chars_traversed));

            chars_traversed += line.len() + 1;
            tokens.append(&mut new_tokens);
        }

        tokens
    }
}

fn parse_line(
    source: &[char],
    parser: &mut parsers::Mask<TreeSitterMasker, PlainEnglish>,
) -> Vec<Token> {
    let actual = without_leading(source);

    if actual.is_empty() {
        return Vec::new();
    }

    let source = actual.get_content(source);

    let mut new_tokens = parser.parse(source);

    new_tokens
        .iter_mut()
        .for_each(|t| t.span.push_by(actual.start));

    new_tokens
}

fn without_leading(source: &[char]) -> Span {
    // Skip over the comment start characters
    let actual_start = source
        .iter()
        .position(|c| !c.is_whitespace())
        .unwrap_or(source.len());

    // Chop off the end
    let actual_end = source.len()
        - source
            .iter()
            .rev()
            .position(|c| !c.is_whitespace())
            .unwrap_or(0);

    Span::new(actual_start, actual_end)
}

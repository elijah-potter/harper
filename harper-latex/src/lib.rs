use harper_core::parsers::{self, Parser, PlainEnglish};
use harper_core::Token;
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct LatexParser {
    inner: parsers::Mask<TreeSitterMasker, PlainEnglish>
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
                PlainEnglish
            )
        }
    }
}

impl Parser for LatexParser {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}

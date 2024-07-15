use harper_core::parsers::{self, Parser, PlainEnglish};
use harper_core::Token;
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct HtmlParser {
    inner: parsers::Mask<TreeSitterMasker, PlainEnglish>
}

impl HtmlParser {
    pub fn new() -> Self {
        Self {
            inner: parsers::Mask::new(
                TreeSitterMasker::new(tree_sitter_html::language(), Self::node_condition),
                PlainEnglish
            )
        }
    }

    fn node_condition(n: &Node) -> bool {
        n.kind() == "text"
    }
}

impl Parser for HtmlParser {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}

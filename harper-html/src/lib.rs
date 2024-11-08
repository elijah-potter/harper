use harper_data::Token;
use harper_parsing::{MaskParser, Parser, PlainEnglish};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct HtmlParser {
    inner: MaskParser<TreeSitterMasker, PlainEnglish>,
}

impl HtmlParser {
    fn node_condition(n: &Node) -> bool {
        n.kind() == "text"
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self {
            inner: MaskParser::new(
                TreeSitterMasker::new(tree_sitter_html::language(), Self::node_condition),
                PlainEnglish,
            ),
        }
    }
}

impl Parser for HtmlParser {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}

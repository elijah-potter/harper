use harper_core::{parsers::Parser, Token};
use harper_html::HtmlParser;

use super::without_initiators;

#[derive(Default)]
pub struct JavaDoc {
    html_parser: HtmlParser,
}

impl Parser for JavaDoc {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let actual = without_initiators(source);
        let actual_source = actual.get_content(source);

        let mut tokens = self.html_parser.parse(actual_source);

        for token in tokens.iter_mut() {
            token.span.push_by(actual.start);
        }

        tokens
    }
}

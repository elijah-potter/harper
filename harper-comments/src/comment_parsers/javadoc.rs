use std::collections::VecDeque;

use harper_core::parsers::Parser;
use harper_core::{Punctuation, Token, TokenKind, VecExt};
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

        // We need to remove leading spaces and stars from the block of tokens.
        let mut remove_these: VecDeque<usize> = VecDeque::new();

        let mut cursor = 0;

        while cursor < tokens.len() {
            let maybe_newline = tokens[cursor];

            if let TokenKind::Newline(_) = maybe_newline.kind {
                cursor += 1;

                loop {
                    if cursor >= tokens.len() {
                        break;
                    }

                    let maybe_removable = tokens[cursor];

                    if matches!(
                        maybe_removable.kind,
                        TokenKind::Punctuation(Punctuation::Star) | TokenKind::Space(_)
                    ) {
                        remove_these.push_back(cursor);
                        cursor += 1;
                    } else {
                        break;
                    }
                }
            } else {
                cursor += 1;
            }
        }

        tokens.remove_indices(remove_these);

        for token in tokens.iter_mut() {
            token.span.push_by(actual.start);
        }

        tokens
    }
}

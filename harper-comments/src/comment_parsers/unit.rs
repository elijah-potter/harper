use harper_core::parsers::{Markdown, Parser};
use harper_core::{Span, Token};

use super::without_initiators;

/// A comment parser that strips starting `/` and `*` characters.
///
/// It is meant to cover _most_ cases in _most_ programming languages.
///
/// It assumes it is being provided a single line of comment at a time,
/// including the comment initiation characters.
pub struct Unit;

impl Parser for Unit {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut tokens = Vec::new();

        let mut chars_traversed = 0;
        let mut in_code_fence = false;

        for line in source.split(|c| *c == '\n') {
            if line_is_code_fence(line) {
                in_code_fence = !in_code_fence;
            }

            if in_code_fence {
                chars_traversed += line.len() + 1;
                continue;
            }

            let mut new_tokens = parse_line(line);

            if chars_traversed + line.len() < source.len() {
                new_tokens.push(Token::new(
                    Span::new_with_len(line.len(), 1),
                    harper_core::TokenKind::Newline(1),
                ));
            }

            new_tokens
                .iter_mut()
                .for_each(|t| t.span.push_by(chars_traversed));

            chars_traversed += line.len() + 1;
            tokens.append(&mut new_tokens);
        }

        tokens
    }
}

fn parse_line(source: &[char]) -> Vec<Token> {
    let actual = without_initiators(source);

    if actual.is_empty() {
        return Vec::new();
    }

    let source = actual.get_content(source);

    let mut markdown_parser = Markdown;

    let mut new_tokens = markdown_parser.parse(source);

    new_tokens
        .iter_mut()
        .for_each(|t| t.span.push_by(actual.start));

    new_tokens
}

fn line_is_code_fence(source: &[char]) -> bool {
    let actual = without_initiators(source);
    let actual_chars = actual.get_content(source);

    matches!(actual_chars, ['`', '`', '`', ..])
}

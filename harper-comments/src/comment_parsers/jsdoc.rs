use harper_core::parsers::{Markdown, Parser};
use harper_core::{Punctuation, Token, TokenKind};
use itertools::Itertools;

use super::without_initiators;

pub struct JsDoc;

impl Parser for JsDoc {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        dbg!();

        let actual = without_initiators(source);

        if actual.is_empty() {
            return Vec::new();
        }

        let source = actual.get_content(source);
        let mut tokens = Markdown.parse(source);

        let mut cursor = 0;

        // Handle inline tags
        loop {
            if cursor >= tokens.len() {
                break;
            }

            if let Some(new_cursor) = &tokens[cursor..]
                .iter()
                .position(|t| t.kind == TokenKind::Punctuation(Punctuation::OpenCurly))
                .map(|i| i + cursor)
            {
                cursor = *new_cursor;
            } else {
                break;
            }

            let parsers = [parse_link, parse_tutorial];

            for parser in parsers {
                if let Some(p) = parser(&tokens[cursor..], source) {
                    for tok in &mut tokens[cursor..cursor + p] {
                        tok.kind = TokenKind::Unlintable;
                    }

                    cursor += p;
                    continue;
                }
            }
        }

        // Handle the block tag, if it exists
        if let Some(tag_start) = tokens.iter().tuple_windows().position(|(a, b)| {
            matches!(
                (a, b),
                (
                    Token {
                        kind: TokenKind::Punctuation(Punctuation::At),
                        ..
                    },
                    Token {
                        kind: TokenKind::Word,
                        ..
                    }
                )
            )
        }) {
            for token in &mut tokens[tag_start..] {
                token.kind = TokenKind::Unlintable;
            }
        }

        for token in tokens.iter_mut() {
            token.span.push_by(actual.start);
        }

        tokens
    }
}

fn parse_link(tokens: &[Token], source: &[char]) -> Option<usize> {
    parse_inline_tag(&['l', 'i', 'n', 'k'], tokens, source)
}

fn parse_tutorial(tokens: &[Token], source: &[char]) -> Option<usize> {
    parse_inline_tag(&['t', 'u', 't', 'o', 'r', 'i', 'a', 'l'], tokens, source)
}

/// Checks if the provided token slice begins with an inline tag, returning it's
/// end if so.
fn parse_inline_tag(tag_name: &[char], tokens: &[Token], source: &[char]) -> Option<usize> {
    if !matches!(
        tokens,
        [
            Token {
                kind: TokenKind::Punctuation(Punctuation::OpenCurly),
                ..
            },
            Token {
                kind: TokenKind::Punctuation(Punctuation::At),
                ..
            },
            Token {
                kind: TokenKind::Word,
                ..
            },
            ..,
        ]
    ) {
        return None;
    }

    dbg!(tokens[2].span.get_content(source));

    if tokens[2].span.get_content(source) != tag_name {
        dbg!();
        return None;
    }

    let mut cursor = 3;

    while !matches!(
        tokens.get(cursor),
        Some(Token {
            kind: TokenKind::Punctuation(Punctuation::CloseCurly),
            ..
        })
    ) {
        cursor += 1;
    }

    Some(cursor + 1)
}

#[cfg(test)]
mod tests {
    use harper_core::{Document, Punctuation, TokenKind};

    use crate::TreeSitterParser;

    #[test]
    fn handles_inline_link() {
        let source = "/** See {@link MyClass} and [MyClass's foo property]{@link MyClass#foo}. */";
        let parser = TreeSitterParser::new_from_language_id("javascript").unwrap();
        let document = Document::new(source, Box::new(parser));

        assert_eq!(
            document.tokens().map(|t| t.kind).collect::<Vec<_>>(),
            vec![
                TokenKind::Word,
                TokenKind::Space(1),
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Space(1),
                TokenKind::Word,
                TokenKind::Space(1),
                TokenKind::Punctuation(Punctuation::OpenSquare),
                TokenKind::Word,
                TokenKind::Space(1),
                TokenKind::Word,
                TokenKind::Space(1),
                TokenKind::Word,
                TokenKind::Punctuation(Punctuation::CloseSquare),
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Punctuation(Punctuation::Period),
                TokenKind::Newline(1),
            ]
        );
    }

    #[test]
    fn handles_class() {
        let source = "/** @class Circle representing a circle. */";
        let parser = TreeSitterParser::new_from_language_id("javascript").unwrap();
        let document = Document::new(source, Box::new(parser));

        assert!(document
            .tokens()
            .all(|t| t.kind.is_unlintable() || t.kind.is_newline()));
    }
}

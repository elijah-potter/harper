use harper_core::parsers::{Markdown, Parser};
use harper_core::{Punctuation, Span, Token, TokenKind};
use itertools::Itertools;

use super::without_initiators;

pub struct JsDoc;

impl Parser for JsDoc {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut tokens = Vec::new();

        let mut chars_traversed = 0;

        for line in source.split(|c| *c == '\n') {
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
    let actual_line = without_initiators(source);

    if actual_line.is_empty() {
        return vec![];
    }

    let source_line = actual_line.get_content(source);

    let mut new_tokens = Markdown.parse(source_line);

    // Handle inline tags
    mark_inline_tags(&mut new_tokens);

    // Handle the block tag, if it exists on the current line.
    if let Some(tag_start) = new_tokens.iter().tuple_windows().position(|(a, b)| {
        matches!(
            (a, b),
            (
                Token {
                    kind: TokenKind::Punctuation(Punctuation::At),
                    ..
                },
                Token {
                    kind: TokenKind::Word(..),
                    ..
                }
            )
        )
    }) {
        for token in &mut new_tokens[tag_start..] {
            token.kind = TokenKind::Unlintable;
        }
    }

    for token in new_tokens.iter_mut() {
        token.span.push_by(actual_line.start);
    }

    new_tokens
}

/// Locate all inline tags (i.e. `{@tag ..}`) and mark them as unlintable
pub(super) fn mark_inline_tags(tokens: &mut [Token]) {
    let mut cursor = 0;

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

        if let Some(p) = parse_inline_tag(&tokens[cursor..]) {
            for tok in &mut tokens[cursor..cursor + p] {
                tok.kind = TokenKind::Unlintable;
            }

            cursor += p;
            continue;
        }
        cursor += 1;
    }
}

/// Checks if the provided token slice begins with an inline tag, returning it's
/// end if so.
fn parse_inline_tag(tokens: &[Token]) -> Option<usize> {
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
                kind: TokenKind::Word(..),
                ..
            },
            ..,
        ]
    ) {
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

    use crate::CommentParser;

    #[test]
    fn escapes_loop() {
        let source = "/** This should _not_cause an infinite loop: {@ */";
        let mut parser = CommentParser::new_from_language_id("javascript").unwrap();
        Document::new_curated(source, &mut parser);
    }

    #[test]
    fn handles_inline_link() {
        let source = "/** See {@link MyClass} and [MyClass's foo property]{@link MyClass#foo}. */";
        let mut parser = CommentParser::new_from_language_id("javascript").unwrap();
        let document = Document::new_curated(source, &mut parser);

        assert!(matches!(
            document
                .tokens()
                .map(|t| t.kind)
                .collect::<Vec<_>>()
                .as_slice(),
            &[
                TokenKind::Word(..),
                TokenKind::Space(1),
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Unlintable,
                TokenKind::Space(1),
                TokenKind::Word(..),
                TokenKind::Space(1),
                TokenKind::Punctuation(Punctuation::OpenSquare),
                TokenKind::Word(..),
                TokenKind::Space(1),
                TokenKind::Word(..),
                TokenKind::Space(1),
                TokenKind::Word(..),
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
            ]
        ));
    }

    #[test]
    fn handles_class() {
        let source = "/** @class Circle representing a circle. */";
        let mut parser = CommentParser::new_from_language_id("javascript").unwrap();
        let document = Document::new_curated(source, &mut parser);

        assert!(document
            .tokens()
            .all(|t| t.kind.is_unlintable() || t.kind.is_newline() || t.kind.is_paragraph_break()));
    }
}

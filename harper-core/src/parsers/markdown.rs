use std::collections::VecDeque;

use super::{Parser, PlainEnglish};
use crate::{Span, Token, TokenKind, TokenStringExt, VecExt};

/// A parser that wraps the [`PlainEnglish`] parser that allows one to parse
/// CommonMark files.
///
/// Will ignore code blocks and tables.
pub struct Markdown;

impl Markdown {
    /// Remove hidden Wikilink target text.
    ///
    /// As in, the stuff to the left of the pipe operator:
    ///
    /// ```markdown
    /// [[Target text|Display Text]]
    /// ```
    fn remove_hidden_wikilink_tokens(tokens: &mut Vec<Token>) {
        let mut to_remove = VecDeque::new();

        for pipe_idx in tokens.iter_pipe_indices() {
            // Locate preceding `[[`
            let mut cursor = pipe_idx - 2;
            let mut open_bracket = None;

            loop {
                let Some((a, b)) = tokens.get(cursor).zip(tokens.get(cursor + 1)) else {
                    break;
                };

                if a.kind.is_newline() {
                    break;
                }

                if a.kind.is_open_square() && b.kind.is_open_square() {
                    open_bracket = Some(cursor);
                    break;
                } else if cursor == 0 {
                    break;
                } else {
                    cursor -= 1;
                }
            }

            // Locate succeeding `[[`
            cursor = pipe_idx + 1;
            let mut close_bracket = None;

            loop {
                let Some((a, b)) = tokens.get(cursor).zip(tokens.get(cursor + 1)) else {
                    break;
                };

                if a.kind.is_newline() {
                    break;
                }

                if a.kind.is_close_square() && b.kind.is_close_square() {
                    close_bracket = Some(cursor);
                    break;
                } else {
                    cursor += 1;
                }
            }

            if let Some(open_bracket_idx) = open_bracket {
                if let Some(close_bracket_idx) = close_bracket {
                    to_remove.extend(open_bracket_idx..=pipe_idx);
                    to_remove.push_back(close_bracket_idx);
                    to_remove.push_back(close_bracket_idx + 1);
                }
            }
        }

        tokens.remove_indices(to_remove);
    }

    /// Remove the brackets from Wikilinks without pipe operators.
    /// For __those__ Wikilinks, see [`Self::remove_hidden_wikilink_tokens`]
    fn remove_wikilink_brackets(tokens: &mut Vec<Token>) {
        let mut to_remove = VecDeque::new();
        let mut open_brackets = None;

        let mut cursor = 0;

        loop {
            let Some((a, b)) = tokens.get(cursor).zip(tokens.get(cursor + 1)) else {
                break;
            };

            if let Some(open_brackets_idx) = open_brackets {
                if a.kind.is_newline() {
                    open_brackets = None;
                    cursor += 1;
                    continue;
                }

                if a.kind.is_close_square() && b.kind.is_close_square() {
                    to_remove.push_back(open_brackets_idx);
                    to_remove.push_back(open_brackets_idx + 1);

                    to_remove.push_back(cursor);
                    to_remove.push_back(cursor + 1);

                    open_brackets = None;
                }
            } else if a.kind.is_open_square() && b.kind.is_open_square() {
                open_brackets = Some(cursor);
            }

            cursor += 1;
        }

        tokens.remove_indices(to_remove);
    }
}

impl Parser for Markdown {
    /// This implementation is quite gross to look at, but it works.
    /// If any issues arise, it would likely help to refactor this out first.
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut english_parser = PlainEnglish;

        let source_str: String = source.iter().collect();
        let md_parser = pulldown_cmark::Parser::new_ext(
            &source_str,
            pulldown_cmark::Options::all()
                .difference(pulldown_cmark::Options::ENABLE_SMART_PUNCTUATION),
        );

        let mut tokens = Vec::new();

        let mut traversed_bytes = 0;
        let mut traversed_chars = 0;

        let mut stack = Vec::new();

        // NOTE: the range spits out __byte__ indices, not char indices.
        // This is why we keep track above.
        for (event, range) in md_parser.into_offset_iter() {
            if range.start > traversed_bytes {
                traversed_chars += source_str[traversed_bytes..range.start].chars().count();
                traversed_bytes = range.start;
            }

            match event {
                pulldown_cmark::Event::SoftBreak => {
                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, 1),
                        kind: TokenKind::Newline(1),
                    });
                }
                pulldown_cmark::Event::HardBreak => {
                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, 1),
                        kind: TokenKind::Newline(2),
                    });
                }
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::List(v)) => {
                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, 0),
                        kind: TokenKind::Newline(2),
                    });
                    stack.push(pulldown_cmark::Tag::List(v));
                }
                pulldown_cmark::Event::Start(tag) => stack.push(tag),
                pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Paragraph)
                | pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Item)
                | pulldown_cmark::Event::End(pulldown_cmark::TagEnd::Heading(_))
                | pulldown_cmark::Event::End(pulldown_cmark::TagEnd::TableCell) => {
                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, 0),
                        kind: TokenKind::Newline(2),
                    });
                    stack.pop();
                }
                pulldown_cmark::Event::End(_) => {
                    stack.pop();
                }
                pulldown_cmark::Event::InlineMath(code)
                | pulldown_cmark::Event::DisplayMath(code)
                | pulldown_cmark::Event::Code(code) => {
                    let chunk_len = code.chars().count();

                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, chunk_len),
                        kind: TokenKind::Unlintable,
                    });
                }
                pulldown_cmark::Event::Text(text) => {
                    let chunk_len = text.chars().count();

                    if let Some(tag) = stack.last() {
                        use pulldown_cmark::Tag;

                        if matches!(tag, Tag::CodeBlock(..)) {
                            tokens.push(Token {
                                span: Span::new_with_len(traversed_chars, text.chars().count()),
                                kind: TokenKind::Unlintable,
                            });
                            continue;
                        }

                        if !(matches!(tag, Tag::Paragraph)
                            || matches!(tag, Tag::Link { .. })
                            || matches!(tag, Tag::Heading { .. })
                            || matches!(tag, Tag::Item)
                            || matches!(tag, Tag::TableCell)
                            || matches!(tag, Tag::Emphasis)
                            || matches!(tag, Tag::Strong)
                            || matches!(tag, Tag::Strikethrough))
                        {
                            continue;
                        }
                    }

                    let mut new_tokens =
                        english_parser.parse(&source[traversed_chars..traversed_chars + chunk_len]);

                    new_tokens
                        .iter_mut()
                        .for_each(|token| token.span.push_by(traversed_chars));

                    tokens.append(&mut new_tokens);
                }
                _ => (),
            }
        }

        if matches!(
            tokens.last(),
            Some(Token {
                kind: TokenKind::Newline(_),
                ..
            })
        ) && source.last() != Some(&'\n')
        {
            tokens.pop();
        }

        Self::remove_hidden_wikilink_tokens(&mut tokens);
        Self::remove_wikilink_brackets(&mut tokens);

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::super::StrParser;
    use super::Markdown;
    use crate::{Punctuation, TokenKind};

    #[test]
    fn survives_emojis() {
        let source = r#"🤷."#;

        Markdown.parse_str(source);
    }

    /// Check whether the Markdown parser will emit a breaking newline
    /// at the end of each input.
    ///
    /// It should _not_ do this.
    #[test]
    fn ends_with_newline() {
        let source = "This is a test.";

        let tokens = Markdown.parse_str(source);
        assert_ne!(tokens.len(), 0);
        assert!(!tokens.last().unwrap().kind.is_newline());
    }

    #[test]
    fn math_becomes_unlintable() {
        let source = r#"$\Katex$ $\text{is}$ $\text{great}$."#;

        let tokens = Markdown.parse_str(source);
        assert_eq!(
            tokens.iter().map(|t| t.kind).collect::<Vec<_>>(),
            vec![
                TokenKind::Unlintable,
                TokenKind::Space(1),
                TokenKind::Unlintable,
                TokenKind::Space(1),
                TokenKind::Unlintable,
                TokenKind::Punctuation(Punctuation::Period)
            ]
        )
    }

    #[test]
    fn hidden_wikilink_text() {
        let source = r#"[[this is hidden|this is not]]"#;

        let tokens = Markdown.parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
            ]
        ))
    }

    #[test]
    fn improper_wikilink_text() {
        let source = r#"this is shown|this is also shown]]"#;

        let tokens = Markdown.parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::Pipe),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::CloseSquare),
                TokenKind::Punctuation(Punctuation::CloseSquare),
            ]
        ))
    }

    #[test]
    fn normal_wikilink() {
        let source = r#"[[Wikilink]]"#;
        let tokens = Markdown.parse_str(source);
        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(token_kinds.as_slice(), &[TokenKind::Word(_)]))
    }
}

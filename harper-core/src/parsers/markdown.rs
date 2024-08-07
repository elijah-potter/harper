use super::{Parser, PlainEnglish};
use crate::{Span, Token, TokenKind};

/// A parser that wraps the [`PlainEnglish`] parser that allows one to parse
/// CommonMark files.
///
/// Will ignore code blocks and tables.
pub struct Markdown;

impl Parser for Markdown {
    /// This implementation is quite gross to look at, but it works.
    /// If any issues arise, it would likely help to refactor this out first.
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut english_parser = PlainEnglish;

        let source_str: String = source.iter().collect();
        let md_parser =
            pulldown_cmark::Parser::new_ext(&source_str, pulldown_cmark::Options::all());

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
                        kind: TokenKind::Newline(1)
                    });
                }
                pulldown_cmark::Event::HardBreak => {
                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, 1),
                        kind: TokenKind::Newline(2)
                    });
                }
                pulldown_cmark::Event::Start(pulldown_cmark::Tag::List(v)) => {
                    tokens.push(Token {
                        span: Span::new_with_len(traversed_chars, 0),
                        kind: TokenKind::Newline(2)
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
                        kind: TokenKind::Newline(2)
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
                        kind: TokenKind::Unlintable
                    });
                }
                pulldown_cmark::Event::Text(text) => {
                    let chunk_len = text.chars().count();

                    if let Some(tag) = stack.last() {
                        use pulldown_cmark::Tag;

                        if matches!(tag, Tag::CodeBlock(..)) {
                            tokens.push(Token {
                                span: Span::new_with_len(traversed_chars, text.chars().count()),
                                kind: TokenKind::Unlintable
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
                _ => ()
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
}

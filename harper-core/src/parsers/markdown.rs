use super::{Parser, PlainEnglish, StrParser};
use crate::Token;

/// A parser that wraps the [`PlainEnglish`] parser that allows one to parse CommonMark files.
///
/// Will ignore code blocks and tables.
pub struct Markdown;

impl Parser for Markdown {
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
            match event {
                pulldown_cmark::Event::Start(tag) => stack.push(tag),
                pulldown_cmark::Event::End(_) => {
                    stack.pop();
                }
                pulldown_cmark::Event::Text(text) => {
                    traversed_chars += source_str[traversed_bytes..range.start].chars().count();
                    traversed_bytes = range.start;

                    if let Some(tag) = stack.last() {
                        use pulldown_cmark::Tag;

                        if !(matches!(tag, Tag::Paragraph)
                            || matches!(tag, Tag::Heading(_, _, _))
                            || matches!(tag, Tag::Item)
                            || matches!(tag, Tag::TableCell)
                            || matches!(tag, Tag::Emphasis)
                            || matches!(tag, Tag::Strong)
                            || matches!(tag, Tag::Link(..))
                            || matches!(tag, Tag::Strikethrough))
                        {
                            continue;
                        }
                    }

                    let mut new_tokens = english_parser.parse_str(text);

                    new_tokens
                        .iter_mut()
                        .for_each(|token| token.span.offset(traversed_chars));

                    tokens.append(&mut new_tokens);
                }
                _ => (),
            }
        }

        tokens
    }
}

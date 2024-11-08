use harper_data::{Span, Token};

use super::Parser;
use crate::lexing::{lex_token, FoundToken};

/// A parser that will attempt to lex as many tokens a possible,
/// without discrimination and until the end of input.
pub struct PlainEnglish;

impl PlainEnglish {}

impl Parser for PlainEnglish {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut cursor = 0;

        // Lex tokens
        let mut tokens = Vec::new();

        loop {
            if cursor == source.len() {
                return tokens;
            }

            if let Some(FoundToken { token, next_index }) = lex_token(&source[cursor..]) {
                tokens.push(Token {
                    span: Span::new(cursor, cursor + next_index),
                    kind: token,
                });
                cursor += next_index;
            } else {
                panic!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use harper_data::TokenStringExt;

    use super::{Parser, PlainEnglish};

    #[test]
    fn parses_sentences_correctly() {
        let text = "There were three little pigs. They built three little homes.";
        let chars: Vec<char> = text.chars().collect();
        let toks = PlainEnglish.parse(&chars);

        let mut sentence_strs = vec![];

        for sentence in toks.iter_sentences() {
            if let Some(span) = sentence.span() {
                sentence_strs.push(span.get_content_string(&chars));
            }
        }

        assert_eq!(
            sentence_strs,
            vec![
                "There were three little pigs.",
                " They built three little homes."
            ]
        )
    }
}

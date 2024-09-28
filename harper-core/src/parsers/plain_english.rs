use super::Parser;
use crate::lexing::{lex_token, FoundToken};
use crate::{Span, Token};

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

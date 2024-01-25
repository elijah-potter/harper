use super::Parser;
use crate::{
    lexing::{lex_token, FoundToken},
    Span, Token,
};

pub struct PlainEnglishParser;

impl Parser for PlainEnglishParser {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut cursor = 0;
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

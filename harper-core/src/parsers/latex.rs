use super::{Parser, PlainEnglish};
use crate::{Span, Token, TokenKind};

pub struct Latex;

impl Parser for Latex {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut english_parser = PlainEnglish;

        let source_str: String = source.iter().collect();

        todo!()
    }
}

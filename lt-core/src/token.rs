use is_macro::Is;
use serde::{Deserialize, Serialize};

use crate::span::Span;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

#[derive(Debug, Is, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenKind {
    Word,
    Punctuation(Punctuation),
    Number(f64),
    /// A sequence of " " spaces.
    Space(usize),
    /// A sequence of "\n" newlines
    Newline(usize),
}

#[derive(Debug, Is, Clone, Serialize, Deserialize, PartialEq)]
pub enum Punctuation {
    /// .
    Period,
    /// !
    Bang,
    /// ?
    Question,
    /// :
    Colon,
    /// ;
    Semicolon,
    /// "
    Quote,
    /// ,
    Comma,
    /// -
    Hyphen,
    /// ' or ’
    Apostrophe,
    /// [
    OpenSquare,
    /// ]
    CloseSquare,
    /// (
    OpenRound,
    /// )
    CloseRound,
    /// "
    Hash,
}

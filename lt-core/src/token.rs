use is_macro::Is;
use serde::{Deserialize, Serialize};

use crate::span::Span;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

#[derive(Debug, Is, Clone, Serialize, Deserialize)]
pub enum TokenKind {
    Word,
    Punctuation(Punctuation),
    Number(f64),
}

#[derive(Debug, Is, Clone, Serialize, Deserialize)]
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

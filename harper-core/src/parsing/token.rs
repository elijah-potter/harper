use is_macro::Is;
use serde::{Deserialize, Serialize};

use crate::span::Span;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl Token {
    /// Convert to an allocated [`FatToken`].
    pub fn to_fat(&self, source: &[char]) -> FatToken {
        let content = self.span.get_content(source).to_vec();

        FatToken {
            content,
            kind: self.kind,
        }
    }
}

/// A [`Token`] that holds its content as a fat [`Vec<char>`] rather than as a [`Span`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatToken {
    pub content: Vec<char>,
    pub kind: TokenKind,
}

#[derive(Debug, Is, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value")]
pub enum TokenKind {
    Word,
    Punctuation(Punctuation),
    Number(f64),
    /// A sequence of " " spaces.
    Space(usize),
    /// A sequence of "\n" newlines
    Newline(usize),
}

impl TokenKind {
    pub fn as_mut_quote(&mut self) -> Option<&mut Quote> {
        self.as_mut_punctuation()?.as_mut_quote()
    }

    pub fn as_quote(&self) -> Option<&Quote> {
        self.as_punctuation()?.as_quote()
    }
}

#[derive(Debug, Is, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind")]
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
    Quote(Quote),
    /// ,
    Comma,
    /// -
    Hyphen,
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
    /// '
    Apostrophe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quote {
    /// The location of the matching quote, if it exists.
    pub twin_loc: Option<usize>,
}

pub trait TokenStringExt {
    fn first_word(&self) -> Option<Token>;
    fn iter_word_indices(&self) -> impl Iterator<Item = usize> + '_;
    fn iter_words(&self) -> impl Iterator<Item = &Token> + '_;
}

impl TokenStringExt for [Token] {
    fn first_word(&self) -> Option<Token> {
        self.iter().find(|v| v.kind.is_word()).copied()
    }

    fn iter_word_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.iter()
            .enumerate()
            .filter(|(_, t)| t.kind.is_word())
            .map(|(i, _)| i)
    }

    fn iter_words(&self) -> impl Iterator<Item = &Token> + '_ {
        self.iter().filter(|t| t.kind.is_word())
    }
}

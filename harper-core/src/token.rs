use is_macro::Is;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::span::Span;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
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

#[derive(Debug, Is, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(tag = "kind", content = "value")]
pub enum TokenKind {
    #[default]
    Word,
    Punctuation(Punctuation),
    Number(f64),
    /// A sequence of " " spaces.
    Space(usize),
    /// A sequence of "\n" newlines
    Newline(usize),
    /// A special token used for things like inline code blocks that should be ignored by all
    /// linters.
    Unlintable,
}

impl TokenKind {
    pub fn as_mut_quote(&mut self) -> Option<&mut Quote> {
        self.as_mut_punctuation()?.as_mut_quote()
    }

    pub fn as_quote(&self) -> Option<&Quote> {
        self.as_punctuation()?.as_quote()
    }

    pub fn is_apostrophe(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Apostrophe))
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
    /// %
    Percent,
    /// /
    ForwardSlash,
    /// \
    Backslash,
    /// <
    LessThan,
    /// >
    GreaterThan,
    /// Equal
    Equal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quote {
    /// The location of the matching quote, if it exists.
    pub twin_loc: Option<usize>,
}

pub trait TokenStringExt {
    fn first_word(&self) -> Option<Token>;
    /// Grabs the first word in the sentence.
    /// Will also return [`None`] if there is an unlintable token in the position of the first
    /// word.
    fn first_sentence_word(&self) -> Option<Token>;
    fn iter_word_indices(&self) -> impl Iterator<Item = usize> + '_;
    fn iter_words(&self) -> impl Iterator<Item = &Token> + '_;
    fn iter_space_indices(&self) -> impl Iterator<Item = usize> + '_;
    fn iter_spaces(&self) -> impl Iterator<Item = &Token> + '_;
    fn iter_apostrophe_indices(&self) -> impl Iterator<Item = usize> + '_;
    fn iter_apostrophes(&self) -> impl Iterator<Item = &Token> + '_;
    /// Grab the span that represents the beginning of the first element and the end of the last
    /// element.
    fn span(&self) -> Option<Span>;
}

impl TokenStringExt for [Token] {
    fn first_word(&self) -> Option<Token> {
        self.iter().find(|v| v.kind.is_word()).copied()
    }

    fn first_sentence_word(&self) -> Option<Token> {
        let (w_idx, word) = self.iter().find_position(|v| v.kind.is_word())?;

        let Some(u_idx) = self.iter().position(|v| v.kind.is_unlintable()) else {
            return Some(*word);
        };

        if w_idx > u_idx {
            Some(*word)
        } else {
            None
        }
    }

    fn iter_word_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.iter()
            .enumerate()
            .filter(|(_, t)| t.kind.is_word())
            .map(|(i, _)| i)
    }

    fn iter_words(&self) -> impl Iterator<Item = &Token> + '_ {
        self.iter_word_indices().map(|i| &self[i])
    }

    fn iter_space_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.iter()
            .enumerate()
            .filter(|(_, t)| t.kind.is_space())
            .map(|(i, _)| i)
    }

    fn iter_spaces(&self) -> impl Iterator<Item = &Token> + '_ {
        self.iter_space_indices().map(|i| &self[i])
    }

    fn iter_apostrophe_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.iter()
            .enumerate()
            .filter(|(_, t)| t.kind.is_apostrophe())
            .map(|(i, _)| i)
    }

    fn iter_apostrophes(&self) -> impl Iterator<Item = &Token> + '_ {
        self.iter_apostrophe_indices().map(|i| &self[i])
    }

    fn span(&self) -> Option<Span> {
        Some(Span::new(self.first()?.span.start, self.last()?.span.end))
    }
}

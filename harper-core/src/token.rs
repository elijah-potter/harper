use is_macro::Is;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::punctuation::Punctuation;
use crate::span::Span;
use crate::Quote;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind
}

impl Token {
    pub fn new(span: Span, kind: TokenKind) -> Self {
        Self { span, kind }
    }

    /// Convert to an allocated [`FatToken`].
    pub fn to_fat(&self, source: &[char]) -> FatToken {
        let content = self.span.get_content(source).to_vec();

        FatToken {
            content,
            kind: self.kind
        }
    }
}

/// A [`Token`] that holds its content as a fat [`Vec<char>`] rather than as a
/// [`Span`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct FatToken {
    pub content: Vec<char>,
    pub kind: TokenKind
}

#[derive(Debug, Is, Clone, Copy, Serialize, Deserialize, PartialEq, Default, PartialOrd)]
#[serde(tag = "kind", content = "value")]
pub enum TokenKind {
    #[default]
    Word,
    Punctuation(Punctuation),
    Number(f64, Option<NumberSuffix>),
    /// A sequence of " " spaces.
    Space(usize),
    /// A sequence of "\n" newlines
    Newline(usize),
    EmailAddress,
    Url,
    Hostname,
    /// A special token used for things like inline code blocks that should be
    /// ignored by all linters.
    Unlintable
}

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, PartialOrd, Clone, Copy, Is)]
pub enum NumberSuffix {
    #[default]
    Th,
    St,
    Nd,
    Rd
}

impl NumberSuffix {
    pub fn correct_suffix_for(number: f64) -> Option<Self> {
        if number < 0.0 || number - number.floor() > f64::EPSILON || number > u64::MAX as f64 {
            return None;
        }

        let integer = number as u64;

        if let 11..=13 = integer % 100 {
            return Some(Self::Th);
        };

        match integer % 10 {
            0 => Some(Self::Th),
            1 => Some(Self::St),
            2 => Some(Self::Nd),
            3 => Some(Self::Rd),
            4 => Some(Self::Th),
            5 => Some(Self::Th),
            6 => Some(Self::Th),
            7 => Some(Self::Th),
            8 => Some(Self::Th),
            9 => Some(Self::Th),
            _ => None
        }
    }

    pub fn to_chars(self) -> Vec<char> {
        match self {
            NumberSuffix::Th => vec!['t', 'h'],
            NumberSuffix::St => vec!['s', 't'],
            NumberSuffix::Nd => vec!['n', 'd'],
            NumberSuffix::Rd => vec!['r', 'd']
        }
    }

    /// Check the first several characters in a buffer to see if it matches a
    /// number suffix.
    pub fn from_chars(chars: &[char]) -> Option<Self> {
        if chars.len() < 2 {
            return None;
        }

        match (chars[0], chars[1]) {
            ('t', 'h') => Some(NumberSuffix::Th),
            ('T', 'h') => Some(NumberSuffix::Th),
            ('t', 'H') => Some(NumberSuffix::Th),
            ('T', 'H') => Some(NumberSuffix::Th),
            ('s', 't') => Some(NumberSuffix::St),
            ('S', 't') => Some(NumberSuffix::St),
            ('s', 'T') => Some(NumberSuffix::St),
            ('S', 'T') => Some(NumberSuffix::St),
            ('n', 'd') => Some(NumberSuffix::Nd),
            ('N', 'd') => Some(NumberSuffix::Nd),
            ('n', 'D') => Some(NumberSuffix::Nd),
            ('N', 'D') => Some(NumberSuffix::Nd),
            ('r', 'd') => Some(NumberSuffix::Rd),
            ('R', 'd') => Some(NumberSuffix::Rd),
            ('r', 'D') => Some(NumberSuffix::Rd),
            ('R', 'D') => Some(NumberSuffix::Rd),
            _ => None
        }
    }
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

    /// Checks whether the token is whitespace.
    pub fn is_whitespace(&self) -> bool {
        matches!(self, TokenKind::Space(_) | TokenKind::Newline(_))
    }
}

pub trait TokenStringExt {
    fn first_word(&self) -> Option<Token>;
    /// Grabs the first word in the sentence.
    /// Will also return [`None`] if there is an unlintable token in the
    /// position of the first word.
    fn first_sentence_word(&self) -> Option<Token>;
    /// Grabs the first token that isn't whitespace from the token string.
    fn first_non_whitespace(&self) -> Option<Token>;
    fn iter_word_indices(&self) -> impl Iterator<Item = usize> + '_;
    fn iter_words(&self) -> impl Iterator<Item = &Token> + '_;
    fn iter_space_indices(&self) -> impl Iterator<Item = usize> + '_;
    fn iter_spaces(&self) -> impl Iterator<Item = &Token> + '_;
    fn iter_apostrophe_indices(&self) -> impl Iterator<Item = usize> + '_;
    fn iter_apostrophes(&self) -> impl Iterator<Item = &Token> + '_;
    /// Grab the span that represents the beginning of the first element and the
    /// end of the last element.
    fn span(&self) -> Option<Span>;

    fn iter_quote_indices(&self) -> impl Iterator<Item = usize> + '_;
    fn iter_quotes(&self) -> impl Iterator<Item = Token> + '_;
    fn iter_number_indices(&self) -> impl Iterator<Item = usize> + '_;
    fn iter_numbers(&self) -> impl Iterator<Item = Token> + '_;
    fn iter_at_indices(&self) -> impl Iterator<Item = usize> + '_;
    fn iter_at(&self) -> impl Iterator<Item = Token> + '_;
}

impl TokenStringExt for [Token] {
    fn first_word(&self) -> Option<Token> {
        self.iter().find(|v| v.kind.is_word()).copied()
    }

    fn first_non_whitespace(&self) -> Option<Token> {
        self.iter().find(|t| !t.kind.is_whitespace()).copied()
    }

    fn first_sentence_word(&self) -> Option<Token> {
        let (w_idx, word) = self.iter().find_position(|v| v.kind.is_word())?;

        let Some(u_idx) = self.iter().position(|v| v.kind.is_unlintable()) else {
            return Some(*word);
        };

        if w_idx < u_idx {
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

    fn iter_quote_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.iter().enumerate().filter_map(|(idx, token)| {
            if let TokenKind::Punctuation(Punctuation::Quote(_)) = &token.kind {
                Some(idx)
            } else {
                None
            }
        })
    }

    fn iter_quotes(&self) -> impl Iterator<Item = Token> + '_ {
        self.iter_quote_indices().map(|idx| self[idx])
    }

    fn iter_number_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.iter().enumerate().filter_map(|(idx, token)| {
            if let TokenKind::Number(..) = &token.kind {
                Some(idx)
            } else {
                None
            }
        })
    }

    fn iter_numbers(&self) -> impl Iterator<Item = Token> + '_ {
        self.iter_number_indices().map(|idx| self[idx])
    }

    /// Iterates through the indices of all "@" signs.
    fn iter_at_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.iter().enumerate().filter_map(|(idx, token)| {
            if let TokenKind::Punctuation(Punctuation::At) = &token.kind {
                Some(idx)
            } else {
                None
            }
        })
    }

    fn iter_at(&self) -> impl Iterator<Item = Token> + '_ {
        self.iter_at_indices().map(|idx| self[idx])
    }
}

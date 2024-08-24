use is_macro::Is;
use itertools::Itertools;
use paste::paste;
use serde::{Deserialize, Serialize};

use crate::punctuation::Punctuation;
use crate::span::Span;
use crate::{Quote, WordMetadata};

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
    Word(WordMetadata),
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
    #[default]
    Unlintable,
    ParagraphBreak
}

impl TokenKind {
    pub fn is_open_square(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::OpenSquare))
    }

    pub fn is_close_square(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::CloseSquare))
    }

    pub fn is_pipe(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Pipe))
    }

    /// Checks that `self` is the same enum variant as `other`, regardless of
    /// whether the inner metadata is also equal.
    pub fn matches_variant_of(&self, other: &Self) -> bool {
        self.with_default_data() == other.with_default_data()
    }

    /// Produces a copy of `self` with any inner data replaced with it's default
    /// value. Useful for making comparisons on just the variant of the
    /// enum.
    pub fn with_default_data(&self) -> Self {
        match self {
            TokenKind::Word(_) => TokenKind::Word(Default::default()),
            TokenKind::Punctuation(_) => TokenKind::Punctuation(Default::default()),
            TokenKind::Number(..) => TokenKind::Number(Default::default(), Default::default()),
            TokenKind::Space(_) => TokenKind::Space(Default::default()),
            TokenKind::Newline(_) => TokenKind::Newline(Default::default()),
            _ => *self
        }
    }
}

impl TokenKind {
    /// Construct a [`TokenKind::Word`] with no metadata.
    pub fn blank_word() -> Self {
        Self::Word(WordMetadata::default())
    }
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

    pub fn is_quote(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Quote(_)))
    }

    pub fn is_apostrophe(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Apostrophe))
    }

    pub fn is_period(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::Period))
    }

    pub fn is_at(&self) -> bool {
        matches!(self, TokenKind::Punctuation(Punctuation::At))
    }

    /// Checks whether the token is whitespace.
    pub fn is_whitespace(&self) -> bool {
        matches!(self, TokenKind::Space(_) | TokenKind::Newline(_))
    }
}

macro_rules! create_decl_for {
    ($thing:ident) => {
        paste! {
            fn [< first_ $thing >](&self) -> Option<Token> ;

            fn [< last_ $thing >](&self) -> Option<Token> ;

            fn [<iter_ $thing _indices>](&self) -> impl Iterator<Item = usize> + '_;

            fn [<iter_ $thing s>](&self) -> impl Iterator<Item = Token> + '_;
        }
    };
}

macro_rules! create_fns_for {
    ($thing:ident) => {
        paste! {
            fn [< first_ $thing >](&self) -> Option<Token> {
                self.iter().find(|v| v.kind.[<is_ $thing>]()).copied()
            }

            fn [< last_ $thing >](&self) -> Option<Token> {
                self.iter().rev().find(|v| v.kind.[<is_ $thing>]()).copied()
            }

            fn [<iter_ $thing _indices>](&self) -> impl Iterator<Item = usize> + '_ {
                self.iter()
                    .enumerate()
                    .filter(|(_, t)| t.kind.[<is_ $thing>]())
                    .map(|(i, _)| i)
            }

            fn [<iter_ $thing s>](&self) -> impl Iterator<Item = Token> + '_ {
                self.[<iter_ $thing _indices>]().map(|i| self[i])
            }
        }
    };
}

pub trait TokenStringExt {
    fn first_sentence_word(&self) -> Option<Token>;
    fn first_non_whitespace(&self) -> Option<Token>;
    /// Grab the span that represents the beginning of the first element and the
    /// end of the last element.
    fn span(&self) -> Option<Span>;

    create_decl_for!(word);
    create_decl_for!(space);
    create_decl_for!(apostrophe);
    create_decl_for!(pipe);
    create_decl_for!(quote);
    create_decl_for!(number);
    create_decl_for!(at);

    fn iter_linking_verb_indices(&self) -> impl Iterator<Item = usize> + '_;
    fn iter_linking_verbs(&self) -> impl Iterator<Item = Token> + '_;
}

impl TokenStringExt for [Token] {
    create_fns_for!(word);
    create_fns_for!(space);
    create_fns_for!(apostrophe);
    create_fns_for!(pipe);
    create_fns_for!(quote);
    create_fns_for!(number);
    create_fns_for!(at);

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

    fn span(&self) -> Option<Span> {
        Some(Span::new(self.first()?.span.start, self.last()?.span.end))
    }

    fn iter_linking_verb_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.iter_word_indices().filter(|idx| {
            let word = self[*idx];
            let TokenKind::Word(word) = word.kind else {
                panic!("Should be unreachable.");
            };

            word.is_linking_verb()
        })
    }

    fn iter_linking_verbs(&self) -> impl Iterator<Item = Token> + '_ {
        self.iter_linking_verb_indices().map(|idx| self[idx])
    }
}

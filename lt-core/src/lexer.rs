use crate::span::Span;

use super::token::{Punctuation, Token, TokenKind};

#[derive(Debug)]
pub struct FoundToken {
    /// The index of the character __after__ the lexed token
    pub next_index: usize,
    /// Token lexed
    pub token: TokenKind,
}

/// Lex all tokens, if possible.
pub fn lex_to_end(source: &[char]) -> Vec<Token> {
    let mut cursor = 0;
    let mut tokens = Vec::new();

    loop {
        cursor += lex_ignorables(&source[cursor..]);

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
            cursor += 1;
        }
    }
}

fn lex_token(source: &[char]) -> Option<FoundToken> {
    let lexers = [lex_punctuation, lex_number, lex_word];

    for lexer in lexers {
        if let Some(f) = lexer(source) {
            return Some(f);
        }
    }
    None
}

fn lex_word(source: &[char]) -> Option<FoundToken> {
    let mut end = 0;

    while end < source.len() {
        if lex_punctuation(&source[end + 1..]).is_none() && lex_ignorables(&source[end + 1..]) == 0
        {
            end += 1;
        } else {
            return Some(FoundToken {
                next_index: end + 1,
                token: TokenKind::Word,
            });
        }
    }

    None
}

pub fn lex_number(source: &[char]) -> Option<FoundToken> {
    if source.is_empty() {
        return None;
    }

    if !source[0].is_numeric() {
        return None;
    }

    let Some(end) = source
        .iter()
        .enumerate()
        .rev()
        .find_map(|(i, v)| v.is_numeric().then_some(i))
    else {
        return None;
    };

    {
        let s: String = source[0..end + 1].iter().collect();

        if let Ok(n) = s.parse::<f64>() {
            return Some(FoundToken {
                token: TokenKind::Number(n),
                next_index: end + 1,
            });
        }
    }

    lex_number(&source[0..end])
}

/// Find the first token _after_ any characters that can be ignored (whitespace, mostly).
fn lex_ignorables(source: &[char]) -> usize {
    let mut cursor = 0;

    loop {
        let last_cursor = cursor;

        cursor += lex_whitespace(&source[cursor..]);

        if last_cursor == cursor {
            break;
        }
    }

    cursor
}

/// Find the first token _after_ whitespace.
fn lex_whitespace(source: &[char]) -> usize {
    for (index, c) in source.iter().enumerate() {
        if !c.is_whitespace() {
            return index;
        }
    }

    source.len()
}

fn lex_characters(source: &[char], cs: &str, token: TokenKind) -> Option<FoundToken> {
    let sep: Vec<_> = cs.chars().collect();

    if source.get(0..cs.len())? == sep {
        Some(FoundToken {
            token,
            next_index: cs.len(),
        })
    } else {
        None
    }
}

macro_rules! lex_punctuation {
    ($($text:literal => $res:ident),*) => {
        fn lex_punctuation(source: &[char]) -> Option<FoundToken> {
            $(
                if let Some(found) = lex_characters(source, $text, TokenKind::Punctuation(Punctuation::$res)){
                    return Some(found);
                }
            )*

            None
        }
    };
}

lex_punctuation! {
    "." => Period,
    "!" => Bang,
    "?" => Question,
    ":" => Colon,
    ";" => Semicolon,
    "\"" => Quote,
    "," => Comma,
    "-" => Hyphen,
    "’" => Apostrophe,
    "'" => Apostrophe,
    "[" =>  OpenSquare,
    "]" =>  CloseSquare,
    "(" =>  OpenRound,
    ")" =>  CloseRound,
    "#" => Hash
}

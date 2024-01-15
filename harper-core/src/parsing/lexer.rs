use crate::span::Span;

use super::{
    token::{Punctuation, Token, TokenKind},
    Quote,
};

#[derive(Debug)]
pub struct FoundToken {
    /// The index of the character __after__ the lexed token
    pub next_index: usize,
    /// Token lexed
    pub token: TokenKind,
}

pub fn lex_to_end_str(source: impl AsRef<str>) -> Vec<Token> {
    let r = source.as_ref();

    let chars: Vec<_> = r.chars().collect();

    lex_to_end(&chars)
}

/// Lex all tokens, if possible.
pub fn lex_to_end(source: &[char]) -> Vec<Token> {
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

fn lex_token(source: &[char]) -> Option<FoundToken> {
    let lexers = [
        lex_spaces,
        lex_newlines,
        lex_punctuation,
        lex_number,
        lex_word,
    ];

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
        if lex_punctuation(&source[end + 1..]).is_none()
            && lex_spaces(&source[end + 1..]).is_none()
            && lex_newlines(&source[end + 1..]).is_none()
            && end + 1 != source.len()
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

fn lex_newlines(source: &[char]) -> Option<FoundToken> {
    let count = source.iter().take_while(|c| **c == '\n').count();

    if count > 0 {
        Some(FoundToken {
            token: TokenKind::Newline(count),
            next_index: count,
        })
    } else {
        None
    }
}

fn lex_spaces(source: &[char]) -> Option<FoundToken> {
    let count = source.iter().take_while(|c| **c == ' ').count();

    if count > 0 {
        Some(FoundToken {
            token: TokenKind::Space(count),
            next_index: count,
        })
    } else {
        None
    }
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
            if let Some(found) = lex_quote(source){
                return Some(found);
            }

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
    "," => Comma,
    "-" => Hyphen,
    "[" =>  OpenSquare,
    "]" =>  CloseSquare,
    "(" =>  OpenRound,
    ")" =>  CloseRound,
    "#" => Hash
}

fn lex_quote(source: &[char]) -> Option<FoundToken> {
    let c = *source.first()?;

    if c == '\"' || c == '“' || c == '”' {
        Some(FoundToken {
            next_index: 1,
            token: TokenKind::Punctuation(Punctuation::Quote(Quote { twin_loc: None })),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lex_to_end_str, Punctuation,
        TokenKind::{self, *},
    };

    fn assert_tokens_eq(test_str: impl AsRef<str>, expected: &[TokenKind]) {
        let tokens = lex_to_end_str(test_str);
        let kinds: Vec<_> = tokens.into_iter().map(|v| v.kind).collect();

        assert_eq!(&kinds, expected)
    }

    #[test]
    fn single_letter() {
        assert_tokens_eq("a", &[Word])
    }

    #[test]
    fn sentence() {
        assert_tokens_eq(
            "hello world, my friend",
            &[
                Word,
                Space(1),
                Word,
                Punctuation(Punctuation::Comma),
                Space(1),
                Word,
                Space(1),
                Word,
            ],
        )
    }
}

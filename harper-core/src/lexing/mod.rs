mod email_address;

use self::email_address::lex_email_address;
use crate::token::{Punctuation, Quote, TokenKind};

#[derive(Debug)]
pub struct FoundToken {
    /// The index of the character __after__ the lexed token
    pub next_index: usize,
    /// Token lexed
    pub token: TokenKind
}

pub fn lex_token(source: &[char]) -> Option<FoundToken> {
    let lexers = [
        lex_punctuation,
        lex_spaces,
        lex_newlines,
        lex_number,
        lex_email_address,
        lex_word
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
                token: TokenKind::Word
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
                next_index: end + 1
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
            next_index: count
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
            next_index: count
        })
    } else {
        None
    }
}

fn lex_punctuation(source: &[char]) -> Option<FoundToken> {
    if let Some(found) = lex_quote(source) {
        return Some(found);
    }

    let c = source.first()?;

    use Punctuation::*;

    let punct = match c {
        '@' => At,
        '~' => Tilde,
        '=' => Equal,
        '<' => LessThan,
        '>' => GreaterThan,
        '/' => ForwardSlash,
        '\\' => Backslash,
        '%' => Percent,
        '’' => Apostrophe,
        '\'' => Apostrophe,
        '.' => Period,
        '!' => Bang,
        '?' => Question,
        ':' => Colon,
        ';' => Semicolon,
        ',' => Comma,
        '-' => Hyphen,
        '[' => OpenSquare,
        ']' => CloseSquare,
        '{' => OpenCurly,
        '}' => CloseCurly,
        '(' => OpenRound,
        ')' => CloseRound,
        '#' => Hash,
        '*' => Star,
        '&' => Ampersand,
        '–' => EnDash,
        '—' => EmDash,
        '…' => Ellipsis,
        '^' => Carrot,
        '+' => Plus,
        '$' => Dollar,
        '|' => Pipe,
        '_' => Underscore,
        _ => return None
    };

    Some(FoundToken {
        next_index: 1,
        token: TokenKind::Punctuation(punct)
    })
}

fn lex_quote(source: &[char]) -> Option<FoundToken> {
    let c = *source.first()?;

    if c == '\"' || c == '“' || c == '”' {
        Some(FoundToken {
            next_index: 1,
            token: TokenKind::Punctuation(Punctuation::Quote(Quote { twin_loc: None }))
        })
    } else {
        None
    }
}

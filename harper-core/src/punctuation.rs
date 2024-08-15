use is_macro::Is;
use serde::{Deserialize, Serialize};

#[derive(Debug, Is, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Default)]
#[serde(tag = "kind")]
pub enum Punctuation {
    /// `…`
    Ellipsis,
    /// `–`
    EnDash,
    /// `—`
    EmDash,
    /// `&`
    Ampersand,
    /// `.`
    #[default]
    Period,
    /// `!`
    Bang,
    /// `?`
    Question,
    /// `:`
    Colon,
    /// ``;``
    Semicolon,
    /// `"`
    Quote(Quote),
    /// `,`
    Comma,
    /// `-`
    Hyphen,
    /// `[`
    OpenSquare,
    /// `]`
    CloseSquare,
    /// `(`
    OpenRound,
    /// `)`
    CloseRound,
    /// `{`
    OpenCurly,
    /// `}`
    CloseCurly,
    /// `"`
    Hash,
    /// `'`
    Apostrophe,
    /// `%`
    Percent,
    /// `/`
    ForwardSlash,
    /// `\`
    Backslash,
    /// `<`
    LessThan,
    /// `>`
    GreaterThan,
    /// `=`
    Equal,
    /// `*`
    Star,
    /// `~`
    Tilde,
    /// `@`
    At,
    /// `^`
    Carrot,
    /// `+`
    Plus,
    /// `$`
    Dollar,
    /// `|`
    Pipe,
    /// `_`
    Underscore
}

impl Punctuation {
    pub fn from_char(c: char) -> Option<Punctuation> {
        let punct = match c {
            '@' => Punctuation::At,
            '~' => Punctuation::Tilde,
            '=' => Punctuation::Equal,
            '<' => Punctuation::LessThan,
            '>' => Punctuation::GreaterThan,
            '/' => Punctuation::ForwardSlash,
            '\\' => Punctuation::Backslash,
            '%' => Punctuation::Percent,
            '’' => Punctuation::Apostrophe,
            '\'' => Punctuation::Apostrophe,
            '.' => Punctuation::Period,
            '!' => Punctuation::Bang,
            '?' => Punctuation::Question,
            ':' => Punctuation::Colon,
            ';' => Punctuation::Semicolon,
            ',' => Punctuation::Comma,
            '-' => Punctuation::Hyphen,
            '[' => Punctuation::OpenSquare,
            ']' => Punctuation::CloseSquare,
            '{' => Punctuation::OpenCurly,
            '}' => Punctuation::CloseCurly,
            '(' => Punctuation::OpenRound,
            ')' => Punctuation::CloseRound,
            '#' => Punctuation::Hash,
            '*' => Punctuation::Star,
            '&' => Punctuation::Ampersand,
            '–' => Punctuation::EnDash,
            '—' => Punctuation::EmDash,
            '…' => Punctuation::Ellipsis,
            '^' => Punctuation::Carrot,
            '+' => Punctuation::Plus,
            '$' => Punctuation::Dollar,
            '|' => Punctuation::Pipe,
            '_' => Punctuation::Underscore,
            _ => return None
        };

        Some(punct)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd)]
pub struct Quote {
    /// The location of the matching quote, if it exists.
    pub twin_loc: Option<usize>
}

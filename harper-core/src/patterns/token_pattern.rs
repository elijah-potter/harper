use hashbrown::HashSet;
use is_macro::Is;

use super::Pattern;
use crate::{Lrc, Token, TokenKind};

/// A pattern that matches a single token.
#[derive(Debug, Clone, Is)]
pub enum TokenPattern {
    /// Checks that the overall kind of token matches, not whether the inner
    /// data does.
    KindLoose(TokenKind),
    /// Checks that the kind matches exactly.
    KindStrict(TokenKind),
    WhiteSpace,
    WordExact(&'static str),
    WordInSet(Lrc<HashSet<&'static str>>)
}

impl Pattern for TokenPattern {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        let Some(tok) = tokens.first() else {
            return 0;
        };

        match self {
            TokenPattern::KindLoose(kind) => {
                if kind.with_default_data() == tok.kind.with_default_data() {
                    1
                } else {
                    0
                }
            }
            TokenPattern::KindStrict(kind) => {
                if *kind == tok.kind {
                    1
                } else {
                    0
                }
            }
            TokenPattern::WhiteSpace => tokens
                .iter()
                .position(|t| !t.kind.is_whitespace())
                .unwrap_or(tokens.len()),
            TokenPattern::WordExact(word) => {
                if !tok.kind.is_word() {
                    return 0;
                }

                let tok_chars = tok.span.get_content(source);

                let mut w_char_count = 0;
                for (i, w_char) in word.chars().enumerate() {
                    w_char_count += 1;

                    if tok_chars.get(i).cloned() != Some(w_char) {
                        return 0;
                    }
                }

                if w_char_count == tok_chars.len() {
                    1
                } else {
                    0
                }
            }
            TokenPattern::WordInSet(set) => {
                let tok_chars = tok.span.get_content(source);
                let word: String = tok_chars.iter().collect();
                if set.contains(word.as_str()) {
                    1
                } else {
                    0
                }
            }
        }
    }
}

use hashbrown::HashSet;
use paste::paste;

use super::whitespace_pattern::WhitespacePattern;
use super::{Pattern, RepeatingPattern};
use crate::{Lrc, Token, TokenKind};

/// A pattern that checks that a sequence of others patterns match.
#[derive(Default)]
pub struct SequencePattern {
    token_patterns: Vec<Box<dyn Pattern>>,
}

macro_rules! gen_then_from_is {
    ($quality:ident) => {
        paste! {
            pub fn [< then_$quality >] (mut self) -> Self{
                self.token_patterns.push(Box::new(|tok: &Token, _source: &[char]| {
                    tok.kind.[< is_$quality >]()
                }));

                self
            }

            pub fn [< then_one_or_more_$quality s >] (self) -> Self{
                self.then_one_or_more(Box::new(|tok: &Token, _source: &[char]| {
                    tok.kind.[< is_$quality >]()
                }))
            }
        }
    };
}

impl SequencePattern {
    gen_then_from_is!(noun);
    gen_then_from_is!(verb);
    gen_then_from_is!(linking_verb);
    gen_then_from_is!(pronoun);
    gen_then_from_is!(punctuation);
    gen_then_from_is!(conjunction);
    gen_then_from_is!(comma);
    gen_then_from_is!(period);
    gen_then_from_is!(case_separator);
    gen_then_from_is!(adverb);
    gen_then_from_is!(adjective);

    pub fn then_exact_word(mut self, word: &'static str) -> Self {
        self.token_patterns
            .push(Box::new(|tok: &Token, source: &[char]| {
                if !tok.kind.is_word() {
                    return false;
                }

                let tok_chars = tok.span.get_content(source);

                let mut w_char_count = 0;
                for (i, w_char) in word.chars().enumerate() {
                    w_char_count += 1;

                    if tok_chars.get(i).cloned() != Some(w_char) {
                        return false;
                    }
                }

                w_char_count == tok_chars.len()
            }));
        self
    }

    pub fn then_loose(mut self, kind: TokenKind) -> Self {
        self.token_patterns
            .push(Box::new(move |tok: &Token, _source: &[char]| {
                kind.with_default_data() == tok.kind.with_default_data()
            }));

        self
    }

    pub fn then_any_word(mut self) -> Self {
        self.token_patterns
            .push(Box::new(|tok: &Token, _source: &[char]| tok.kind.is_word()));
        self
    }

    pub fn then_strict(mut self, kind: TokenKind) -> Self {
        self.token_patterns
            .push(Box::new(move |tok: &Token, _source: &[char]| {
                tok.kind == kind
            }));
        self
    }

    pub fn then_whitespace(mut self) -> Self {
        self.token_patterns.push(Box::new(WhitespacePattern));
        self
    }

    pub fn then_any_word_in(mut self, word_set: Lrc<HashSet<&'static str>>) -> Self {
        self.token_patterns
            .push(Box::new(move |tok: &Token, source: &[char]| {
                let tok_chars = tok.span.get_content(source);
                let word: String = tok_chars.iter().collect();
                word_set.contains(word.as_str())
            }));
        self
    }

    pub fn then_one_or_more(mut self, pat: Box<dyn Pattern>) -> Self {
        self.token_patterns
            .push(Box::new(RepeatingPattern::new(pat)));
        self
    }

    pub fn then(mut self, pat: Box<dyn Pattern>) -> Self {
        self.token_patterns.push(pat);
        self
    }
}

impl Pattern for SequencePattern {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        let mut tok_cursor = 0;

        for pat in self.token_patterns.iter() {
            let match_length = pat.matches(&tokens[tok_cursor..], source);

            if match_length == 0 {
                return 0;
            }

            tok_cursor += match_length;
        }

        tok_cursor
    }
}

#[cfg(test)]
mod tests {
    use hashbrown::HashSet;

    use super::SequencePattern;
    use crate::patterns::Pattern;
    use crate::{Document, Lrc};

    #[test]
    fn matches_n_whitespace_tokens() {
        let pat = SequencePattern::default()
            .then_any_word()
            .then_whitespace()
            .then_any_word();
        let doc = Document::new_plain_english_curated("word\n    \nword");

        assert_eq!(
            pat.matches(doc.get_tokens(), doc.get_source()),
            doc.get_tokens().len()
        );
    }

    #[test]
    fn matches_specific_words() {
        let pat = SequencePattern::default()
            .then_exact_word("she")
            .then_whitespace()
            .then_exact_word("her");
        let doc = Document::new_plain_english_curated("she her");

        assert_eq!(
            pat.matches(doc.get_tokens(), doc.get_source()),
            doc.get_tokens().len()
        );
    }

    #[test]
    fn matches_sets() {
        let mut pronouns = HashSet::new();
        pronouns.insert("his");
        pronouns.insert("hers");
        let pronouns = Lrc::new(pronouns);

        let pat = SequencePattern::default()
            .then_exact_word("it")
            .then_whitespace()
            .then_exact_word("was")
            .then_whitespace()
            .then_any_word_in(pronouns);
        let doc = Document::new_plain_english_curated("it was hers");

        assert_eq!(
            pat.matches(doc.get_tokens(), doc.get_source()),
            doc.get_tokens().len()
        );
    }
}

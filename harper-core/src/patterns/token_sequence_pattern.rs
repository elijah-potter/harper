use hashbrown::HashSet;

use super::token_pattern::TokenPattern;
use super::Pattern;
use crate::{Lrc, Token, TokenKind};

/// A pattern that checks that a sequence of [`TokenPattern`] matches.
#[derive(Debug, Default)]
pub struct TokenSequencePattern {
    token_patterns: Vec<TokenPattern>
}

impl TokenSequencePattern {
    pub fn then_exact_word(&mut self, word: &'static str) -> &mut Self {
        self.token_patterns.push(TokenPattern::WordExact(word));
        self
    }

    pub fn then_loose(&mut self, kind: TokenKind) -> &mut Self {
        self.token_patterns.push(TokenPattern::KindLoose(kind));
        self
    }

    pub fn then_any_word(&mut self) -> &mut Self {
        self.token_patterns
            .push(TokenPattern::KindLoose(TokenKind::blank_word()));
        self
    }

    pub fn then_strict(&mut self, kind: TokenKind) -> &mut Self {
        self.token_patterns.push(TokenPattern::KindStrict(kind));
        self
    }

    pub fn then_whitespace(&mut self) -> &mut Self {
        self.token_patterns.push(TokenPattern::WhiteSpace);
        self
    }

    pub fn then_any_word_in(&mut self, word_set: Lrc<HashSet<&'static str>>) -> &mut Self {
        self.token_patterns
            .push(TokenPattern::WordInSet(word_set.into()));
        self
    }
}

impl Pattern for TokenSequencePattern {
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

    use super::TokenSequencePattern;
    use crate::patterns::Pattern;
    use crate::{Document, Lrc};

    #[test]
    fn matches_n_whitespace_tokens() {
        let mut pat = TokenSequencePattern::default();
        pat.then_any_word().then_whitespace().then_any_word();
        let doc = Document::new_plain_english_curated("word\n    \nword");

        assert_eq!(
            pat.matches(doc.get_tokens(), doc.get_source()),
            doc.get_tokens().len()
        );
    }

    #[test]
    fn matches_specific_words() {
        let mut pat = TokenSequencePattern::default();
        pat.then_exact_word("she")
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

        let mut pat = TokenSequencePattern::default();
        pat.then_exact_word("it")
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

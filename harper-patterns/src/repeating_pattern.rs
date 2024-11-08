use harper_data::Token;

use super::Pattern;

/// A pattern that will match one or more repetitions of the same pattern.
///
/// Somewhat reminiscent of the `.*` operator in Regex.
pub struct RepeatingPattern {
    inner: Box<dyn Pattern>,
}

impl RepeatingPattern {
    pub fn new(pattern: Box<dyn Pattern>) -> Self {
        Self { inner: pattern }
    }
}

impl Pattern for RepeatingPattern {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        let mut tok_cursor = 0;

        loop {
            let match_len = self.inner.matches(&tokens[tok_cursor..], source);

            if match_len == 0 {
                return tok_cursor;
            } else {
                tok_cursor += match_len;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use harper_parsing::{Parser, PlainEnglish};

    use super::RepeatingPattern;
    use crate::{AnyPattern, Pattern};

    #[test]
    fn matches_anything() {
        let source_str = "This matcher will match the entirety of any document!";
        let source: Vec<_> = source_str.chars().collect();
        let tokens = PlainEnglish.parse(&source);

        let pat = RepeatingPattern::new(Box::new(AnyPattern));

        assert_eq!(pat.matches(&tokens, &source), tokens.len())
    }
}

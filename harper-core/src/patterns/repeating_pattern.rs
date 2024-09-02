use super::Pattern;
use crate::Token;

/// A pattern that will match one or more repetitions of the same pattern.
///
/// Somewhat reminiscent of the `.*` operator in Regex.
pub struct RepeatingPattern {
    inner: Box<dyn Pattern>
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
    use super::RepeatingPattern;
    use crate::patterns::{Pattern, TokenPattern};
    use crate::Document;

    #[test]
    fn matches_anything() {
        let doc = Document::new_plain_english_curated("This matcher will match anything!");
        let pat = RepeatingPattern::new(Box::new(TokenPattern::Any));

        assert_eq!(
            pat.matches(doc.get_tokens(), doc.get_source()),
            doc.get_tokens().len()
        )
    }
}

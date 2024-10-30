use crate::Token;

use super::Pattern;

/// A pattern that returns the value of the first non-zero match in a list.
#[derive(Default)]
pub struct EitherPattern {
    patterns: Vec<Box<dyn Pattern>>,
}

impl EitherPattern {
    pub fn new(patterns: Vec<Box<dyn Pattern>>) -> Self {
        Self { patterns }
    }
}

impl Pattern for EitherPattern {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        for pattern in self.patterns.iter() {
            let match_len = pattern.matches(tokens, source);

            if match_len > 0 {
                return match_len;
            }
        }

        0
    }
}

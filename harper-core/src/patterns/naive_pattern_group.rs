use super::Pattern;
use crate::Token;

/// A naive pattern collection that naively iterates through a list of patterns,
/// returning the first one that matches.
#[derive(Default)]
pub struct NaivePatternGroup {
    patterns: Vec<Box<dyn Pattern>>,
}

impl NaivePatternGroup {
    pub fn push(&mut self, pattern: Box<dyn Pattern>) {
        self.patterns.push(pattern);
    }
}

impl Pattern for NaivePatternGroup {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        self.patterns
            .iter()
            .find_map(|p| {
                let res = p.matches(tokens, source);

                if res != 0 {
                    Some(res)
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }
}

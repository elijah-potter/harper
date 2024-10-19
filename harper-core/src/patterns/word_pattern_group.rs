use hashbrown::HashMap;

use super::naive_pattern_group::NaivePatternGroup;
use super::{Pattern, SequencePattern};
use crate::CharString;

/// A pattern collection to look for patterns that start with a specific
/// word.
#[derive(Default)]
pub struct WordPatternGroup<P>
where
    P: Pattern,
{
    patterns: HashMap<CharString, P>,
}

impl WordPatternGroup<NaivePatternGroup> {
    pub fn add(&mut self, word: &str, pat: Box<dyn Pattern>) {
        let chars = word.chars().collect();

        if let Some(group) = self.patterns.get_mut(&chars) {
            group.push(pat);
        } else {
            let mut group = NaivePatternGroup::default();
            group.push(pat);
            self.patterns.insert(chars, group);
        }
    }

    /// Add a pattern that matches just a word on its own, without anything else required to match.
    pub fn add_word(&mut self, word: &'static str) {
        self.add(
            word,
            Box::new(SequencePattern::default().then_exact_word(word)),
        );
    }
}

impl<P> Pattern for WordPatternGroup<P>
where
    P: Pattern,
{
    fn matches(&self, tokens: &[crate::Token], source: &[char]) -> usize {
        let Some(first) = tokens.first() else {
            return 0;
        };

        if !first.kind.is_word() {
            return 0;
        }

        let word_chars = first.span.get_content(source);
        let Some(inner_pattern) = self.patterns.get(word_chars) else {
            return 0;
        };

        inner_pattern.matches(tokens, source)
    }
}

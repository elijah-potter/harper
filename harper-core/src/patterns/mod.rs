use crate::Token;

mod naive_pattern_group;
mod token_kind_pattern_group;
mod token_pattern;
mod token_sequence_pattern;
mod word_pattern_group;

pub use naive_pattern_group::NaivePatternGroup;
pub use token_kind_pattern_group::TokenKindPatternGroup;
pub use token_pattern::TokenPattern;
pub use token_sequence_pattern::TokenSequencePattern;
pub use word_pattern_group::WordPatternGroup;

pub trait Pattern {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize;
}

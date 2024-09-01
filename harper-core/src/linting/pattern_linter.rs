use super::{Lint, Linter};
use crate::patterns::Pattern;
use crate::Token;

pub trait PatternLinter {
    /// A simple getter for the pattern to be searched for.
    fn pattern(&self) -> &dyn Pattern;
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint;
}

impl<L> Linter for L
where
    L: PatternLinter + Send + Sync
{
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let source = document.get_source();

        for chunk in document.chunks() {
            for i in 0..chunk.len() {
                let match_len = self.pattern().matches(&chunk[i..], source);

                if match_len != 0 {
                    let lint = self.match_to_lint(&chunk[i..i + match_len], source);

                    lints.push(lint);
                }
            }
        }

        lints
    }
}

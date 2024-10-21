use super::{Lint, Linter};
use crate::patterns::Pattern;
use crate::{Token, TokenStringExt};

#[cfg(not(feature = "concurrent"))]
pub trait PatternLinter {
    /// A simple getter for the pattern to be searched for.
    fn pattern(&self) -> &dyn Pattern;
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint;
}

#[cfg(feature = "concurrent")]
pub trait PatternLinter: Send + Sync {
    /// A simple getter for the pattern to be searched for.
    fn pattern(&self) -> &dyn Pattern;
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint;
}

impl<L> Linter for L
where
    L: PatternLinter,
{
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let source = document.get_source();

        for chunk in document.iter_chunks() {
            let mut tok_cursor = 0;

            loop {
                if tok_cursor >= chunk.len() {
                    break;
                }

                let match_len = self.pattern().matches(&chunk[tok_cursor..], source);

                if match_len != 0 {
                    let lint =
                        self.match_to_lint(&chunk[tok_cursor..tok_cursor + match_len], source);

                    lints.push(lint);
                    tok_cursor += match_len;
                } else {
                    tok_cursor += 1;
                }
            }
        }

        lints
    }
}

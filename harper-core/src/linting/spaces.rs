use super::{Lint, Linter};
use crate::{parsing::TokenStringExt, Document, LintKind, Suggestion, TokenKind};

#[derive(Debug, Default)]
pub struct Spaces;

impl Linter for Spaces {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut output = Vec::new();

        for sentence in document.sentences() {
            for space in sentence.iter_spaces() {
                let TokenKind::Space(count) = space.kind else {
                    panic!("The space iterator should only return spaces.")
                };

                if count > 1 {
                    output.push(Lint {
                        span: space.span,
                        lint_kind: LintKind::Formatting,
                        suggestions: vec![Suggestion::ReplaceWith(vec![' '])],
                        message: format!(
                            "There are {} spaces where there should be only one.",
                            count
                        ),
                    })
                }
            }
        }

        output
    }
}

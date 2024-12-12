use itertools::Itertools;

use super::{Lint, LintKind, LintSeverity, Linter, Suggestion};
use crate::TokenStringExt;

/// A linter that checks that an ellipsis doesn't contain too many periods (or
/// too few).
#[derive(Debug, Default)]
pub struct EllipsisLength;

impl Linter for EllipsisLength {
    fn lint(
        &mut self,
        document: &crate::Document,
        severity: Option<LintSeverity>,
    ) -> Vec<super::Lint> {
        let mut lints = Vec::new();

        for tok in document.iter_ellipsiss() {
            let tok_content = document.get_span_content(tok.span);

            if tok_content.is_empty() {
                continue;
            }

            if tok_content.first().cloned() == Some('.')
                && tok_content.iter().all_equal()
                && tok_content.len() != 3
            {
                lints.push(Lint {
                    span: tok.span,
                    lint_kind: LintKind::Formatting,
                    suggestions: vec![Suggestion::ReplaceWith(vec!['.', '.', '.'])],
                    message: "Horizontal ellipsis must have 3 dots.".to_string(),
                    priority: 31,
                    severity,
                })
            }
        }

        lints
    }
}

#[cfg(test)]
mod tests {
    use super::EllipsisLength;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn allows_correct_ellipsis() {
        assert_lint_count("...", EllipsisLength, 0);
    }

    #[test]
    fn corrects_long_ellipsis() {
        assert_lint_count(".....", EllipsisLength, 1);
        assert_suggestion_result(".....", EllipsisLength, "...");
    }

    #[test]
    fn corrects_short_ellipsis() {
        assert_lint_count("..", EllipsisLength, 1);
        assert_suggestion_result("..", EllipsisLength, "...");
    }
}

use super::{Lint, LintKind, Linter, Suggestion};
use crate::token::TokenStringExt;
use crate::{Document, Span, TokenKind};

/// Detect and warn that the sentence is too long.
#[derive(Debug, Clone, Copy, Default)]
pub struct NumberSuffixCapitalization;

impl Linter for NumberSuffixCapitalization {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut output = Vec::new();

        for number_tok in document.iter_numbers() {
            if let TokenKind::Number(_, None) = number_tok.kind {
                continue;
            }

            let suffix_span = Span::new_with_len(number_tok.span.end, 2).pulled_by(2);
            let chars = document.get_span_content(suffix_span);

            if chars.iter().any(|c| !c.is_lowercase()) {
                output.push(Lint {
                    span: suffix_span,
                    lint_kind: LintKind::Capitalization,
                    message: "This suffix should be lowercase".to_string(),
                    suggestions: vec![Suggestion::ReplaceWith(
                        chars.iter().map(|c| c.to_ascii_lowercase()).collect(),
                    )],
                    ..Default::default()
                })
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::NumberSuffixCapitalization;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn detects_uppercase_suffix() {
        assert_lint_count("2ND", NumberSuffixCapitalization, 1);
    }

    #[test]
    fn detects_inconsistent_suffix() {
        assert_lint_count("2nD", NumberSuffixCapitalization, 1);
    }

    #[test]
    fn passes_correct_case() {
        assert_lint_count("2nd", NumberSuffixCapitalization, 0);
    }
}

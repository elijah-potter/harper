use super::{Lint, LintKind, Linter, Suggestion};
use crate::token::TokenStringExt;
use crate::{CharStringExt, Document, Span};

#[derive(Debug, Clone, Default)]
pub struct RepeatedWords;

impl Linter for RepeatedWords {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            let mut iter = chunk.iter_word_indices().zip(chunk.iter_words()).peekable();

            while let (Some((idx_a, tok_a)), Some((idx_b, tok_b))) = (iter.next(), iter.peek()) {
                let word_a = document.get_span_content(tok_a.span);
                let word_b = document.get_span_content(tok_b.span);

                if !tok_a.kind.is_likely_homograph() && word_a.to_lower() == word_b.to_lower() {
                    let intervening_tokens = &chunk[idx_a + 1..*idx_b];

                    if intervening_tokens.iter().any(|t| !t.kind.is_whitespace()) {
                        continue;
                    }

                    lints.push(Lint {
                        span: Span::new(tok_a.span.start, tok_b.span.end),
                        lint_kind: LintKind::Repetition,
                        suggestions: vec![Suggestion::ReplaceWith(
                            document.get_span_content(tok_a.span).to_vec(),
                        )],
                        message: "Did you mean to repeat this word?".to_string(),
                        ..Default::default()
                    })
                }
            }
        }

        lints
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::assert_lint_count;
    use super::RepeatedWords;

    #[test]
    fn catches_basic() {
        assert_lint_count("I wanted the the banana.", RepeatedWords::default(), 1)
    }

    #[test]
    fn does_not_lint_homographs_address() {
        assert_lint_count("To address address problems.", RepeatedWords::default(), 0);
    }

    #[test]
    fn does_not_lint_homographs_record() {
        assert_lint_count("To record record profits.", RepeatedWords::default(), 0);
    }
}

use super::{Lint, LintKind, Linter, Suggestion};
use crate::token::{Token, TokenKind, TokenStringExt};
use crate::{Document, Span};

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

                if word_a == word_b {
                    let intervening_tokens = &chunk[idx_a + 1..*idx_b];

                    if intervening_tokens.iter().any(|t| !t.kind.is_whitespace()) {
                        continue;
                    }

                    // Detect and remove the whitespace between the repetitions.
                    let remove_end = tok_b.span.end;

                    let remove_start = if let Some(Token {
                        span,
                        kind: TokenKind::Space(_),
                    }) = intervening_tokens.last()
                    {
                        span.start
                    } else {
                        tok_b.span.start
                    };

                    lints.push(Lint {
                        span: Span::new(remove_start, remove_end),
                        lint_kind: LintKind::Repetition,
                        suggestions: vec![Suggestion::Remove],
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
}

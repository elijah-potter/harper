use crate::{NounData, TokenKind, TokenStringExt, WordMetadata};

use super::{Lint, LintKind, Linter, Suggestion};

/// A super-simple linter that makes sure you capitalize "I".
#[derive(Default)]
pub struct CapitalizePersonalPronouns;

impl Linter for CapitalizePersonalPronouns {
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for tok in document.iter_words() {
            if let TokenKind::Word(WordMetadata {
                noun:
                    Some(NounData {
                        is_pronoun: Some(true),
                        ..
                    }),
                ..
            }) = tok.kind
            {
                if document.get_span_content(tok.span) == ['i'] {
                    lints.push(Lint {
                        span: tok.span,
                        lint_kind: LintKind::Capitalization,
                        suggestions: vec![Suggestion::ReplaceWith(vec!['I'])],
                        message: "First-person singular pronouns must be capitalized.".to_string(),
                        priority: 31,
                    });
                }
            }
        }

        lints
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::CapitalizePersonalPronouns;

    #[test]
    fn start() {
        assert_suggestion_result("i am hungry", CapitalizePersonalPronouns, "I am hungry");
    }

    #[test]
    fn end() {
        assert_suggestion_result(
            "There is no one stronger than i",
            CapitalizePersonalPronouns,
            "There is no one stronger than I",
        );
    }

    #[test]
    fn middle() {
        assert_suggestion_result(
            "First of all, i am not happy with this.",
            CapitalizePersonalPronouns,
            "First of all, I am not happy with this.",
        );
    }
}

use super::{Lint, LintKind, Linter};
use crate::token::TokenStringExt;
use crate::Document;

/// Detect and warn that the sentence is too long.
#[derive(Debug, Clone, Copy, Default)]
pub struct LinkingVerbs;

impl Linter for LinkingVerbs {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut output = Vec::new();

        for chunk in document.iter_chunks() {
            // The word prior to "is" must be a noun.
            for idx in chunk.iter_linking_verb_indices() {
                let linking_verb = chunk[idx];
                let linking_verb_text = document.get_span_content_str(linking_verb.span);

                if let Some(prev_word) = &chunk[0..idx].last_word() {
                    if !prev_word.kind.as_word().unwrap().is_noun() {
                        output.push(Lint {
                            span: linking_verb.span,
                            lint_kind: LintKind::Miscellaneous,
                            message: format!(
                                "Linking verbs like “{}” must be preceded by a noun.",
                                linking_verb_text
                            ),
                            ..Default::default()
                        })
                    }
                }
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::LinkingVerbs;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn dora() {
        assert_lint_count("Dora is a noun.", LinkingVerbs, 0);
    }

    #[test]
    fn working_wrong() {
        assert_lint_count("working is not a noun.", LinkingVerbs, 1);
    }

    #[test]
    fn working_right() {
        assert_lint_count("\"working\" is a noun.", LinkingVerbs, 0);
    }
}

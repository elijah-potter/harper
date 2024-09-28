use super::{Lint, LintKind, Linter};
use crate::{Document, TokenStringExt};

#[derive(Debug, Default)]
pub struct AvoidCurses;

impl Linter for AvoidCurses {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        document
            .iter_words()
            .filter(|t| t.kind.is_swear())
            .map(|t| Lint {
                span: t.span,
                lint_kind: LintKind::Miscellaneous,
                suggestions: vec![],
                message: "Try to avoid offensive language.".to_string(),
                priority: 63,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::AvoidCurses;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn detects_shit() {
        assert_lint_count("He ate shit when he fell off the bike.", AvoidCurses, 1);
    }
}

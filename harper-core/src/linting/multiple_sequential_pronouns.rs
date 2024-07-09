use crate::{CharString, Document, Lint, LintKind, Linter, Span, Token, TokenStringExt};

/// Linter that checks if multiple pronouns are being used right after each
/// other. This is a common mistake to make during the revision process.
#[derive(Debug)]
pub struct MultipleSequentialPronouns {
    /// Since there aren't many pronouns, it's faster to store this as a vector.
    pronouns: Vec<CharString>
}

impl MultipleSequentialPronouns {
    fn new() -> Self {
        let pronoun_strs = [
            "me", "my", "I", "we", "you", "he", "him", "her", "she", "it", "they"
        ];

        let mut pronouns: Vec<CharString> = pronoun_strs
            .iter()
            .map(|s| s.chars().collect::<CharString>())
            .collect();

        pronouns.sort();

        Self { pronouns }
    }

    fn is_pronoun(&self, word: &[char]) -> bool {
        self.pronouns
            .binary_search_by_key(&word, |w| w.as_slice())
            .is_ok()
    }
}

impl Linter for MultipleSequentialPronouns {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        let mut found_pronouns = Vec::new();

        let mut emit_lint = |found_pronouns: &mut Vec<&Token>| {
            let first: &&Token = found_pronouns.first().unwrap();
            let last: &&Token = found_pronouns.last().unwrap();

            lints.push(Lint {
                span: Span::new(first.span.start, last.span.end),
                lint_kind: LintKind::Repetition,
                message: "There are too many personal pronouns in sequence here.".to_owned(),
                priority: 63,
                ..Default::default()
            });
            found_pronouns.clear();
        };

        for chunk in document.chunks() {
            for word in chunk.iter_words() {
                let word_chars = document.get_span_content(word.span);

                if self.is_pronoun(word_chars) {
                    found_pronouns.push(word);
                } else if found_pronouns.len() == 1 {
                    found_pronouns.clear();
                } else if found_pronouns.len() > 1 {
                    emit_lint(&mut found_pronouns);
                }
            }

            if found_pronouns.len() > 1 {
                emit_lint(&mut found_pronouns);
            }

            found_pronouns.clear();
        }

        lints
    }
}

impl Default for MultipleSequentialPronouns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::MultipleSequentialPronouns;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn can_detect_two_pronouns() {
        assert_lint_count(
            "...little bit about my I want to do.",
            MultipleSequentialPronouns::new(),
            1
        )
    }

    #[test]
    fn can_detect_three_pronouns() {
        assert_lint_count(
            "...little bit about my I you want to do.",
            MultipleSequentialPronouns::new(),
            1
        )
    }

    #[test]
    fn allows_single_pronouns() {
        assert_lint_count(
            "...little bit about I want to do.",
            MultipleSequentialPronouns::new(),
            0
        )
    }

    #[test]
    fn detects_multiple_pronouns_at_end() {
        assert_lint_count(
            "...little bit about I want to do to me you.",
            MultipleSequentialPronouns::new(),
            1
        )
    }

    #[test]
    fn comma_separated() {
        assert_lint_count("To prove it, we...", MultipleSequentialPronouns::new(), 0)
    }
}

use hashbrown::HashSet;

use super::{Lint, LintKind, Linter};
use crate::token::{Token, TokenKind, TokenStringExt};
use crate::{CharString, Document, Span, Suggestion};

#[derive(Debug, Clone)]
pub struct RepeatedWords {
    /// The set of words that can be considered for repetition checking.
    set: HashSet<CharString>
}

impl RepeatedWords {
    pub fn new() -> Self {
        let mut set = HashSet::new();

        macro_rules! add_set {
            ($lit:literal) => {
                set.insert($lit.chars().collect());
            };
            ($($lit:literal),*) => {
                $(
                    add_set!($lit);
                )*
            }
        }

        add_set!(
            "the", "be", "to", "of", "and", "a", "in", "that", "have", "I", "it", "for", "not",
            "on", "with", "he", "as", "you", "do", "at", "this", "but", "his", "by", "from",
            "they", "we", "say", "her", "she", "or", "an", "will", "my", "one", "all", "would",
            "there", "their", "what", "so", "up", "out", "if", "about", "who", "get", "which",
            "go", "me", "when", "make", "can", "like", "time", "no", "just", "him", "know", "take",
            "people", "into", "year", "your", "good", "some", "could", "them", "see", "other",
            "than", "then", "now", "look", "only", "come", "its", "over", "think", "also", "back",
            "after", "use", "two", "how", "our", "work", "first", "well", "way", "even", "new",
            "want", "because", "any", "these", "give", "day", "most", "us", "are"
        );

        Self { set }
    }
}

impl Default for RepeatedWords {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for RepeatedWords {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.sentences() {
            let mut iter = sentence
                .iter_word_indices()
                .zip(sentence.iter_words())
                .peekable();

            while let (Some((idx_a, tok_a)), Some((idx_b, tok_b))) = (iter.next(), iter.peek()) {
                let word_a = document.get_span_content(tok_a.span);
                let word_b = document.get_span_content(tok_b.span);

                if self.set.contains(word_a) && word_a == word_b {
                    let intervening_tokens = &sentence[idx_a + 1..*idx_b];

                    if intervening_tokens.iter().any(|t| !t.kind.is_whitespace()) {
                        continue;
                    }

                    // Detect and remove the whitespace between the repetitions.
                    let remove_end = tok_b.span.end;

                    let remove_start = if let Some(Token {
                        span,
                        kind: TokenKind::Space(_)
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
        assert_lint_count("I wanted the the banana.", RepeatedWords::new(), 1)
    }
}

use hashbrown::HashSet;

use crate::{
    parsing::{Token, TokenKind, TokenStringExt},
    Document, Span, Suggestion,
};

use super::{Lint, LintKind, Linter};

#[derive(Debug, Clone)]
pub struct RepeatedWords {
    /// The set of words that can be considered for repetition checking.
    set: HashSet<Vec<char>>,
}

impl RepeatedWords {
    pub fn new() -> Self {
        let mut set = HashSet::new();

        set.insert(vec!['t', 'h', 'e']);
        set.insert(vec!['T', 'h', 'e']);
        set.insert(vec!['a']);
        set.insert(vec!['A']);
        set.insert(vec!['a', 'n']);
        set.insert(vec!['A', 'n']);
        set.insert(vec!['i', 's']);
        set.insert(vec!['I', 's']);
        set.insert(vec!['w', 'i', 'l', 'l']);
        set.insert(vec!['W', 'i', 'l', 'l']);
        set.insert(vec!['l', 'i', 'k', 'e']);
        set.insert(vec!['L', 'i', 'k', 'e']);
        set.insert(vec!['t', 'h', 'a', 't']);
        set.insert(vec!['T', 'h', 'a', 't']);
        set.insert(vec!['w', 'h', 'a', 't']);
        set.insert(vec!['W', 'h', 'a', 't']);
        set.insert(vec!['w', 'h', 'i', 'c', 'h']);
        set.insert(vec!['W', 'h', 'i', 'c', 'h']);
        set.insert(vec!['b', 'e']);
        set.insert(vec!['B', 'e']);
        set.insert(vec!['a', 'n', 'd']);
        set.insert(vec!['A', 'n', 'd']);
        set.insert(vec!['I']);
        set.insert(vec!['a', 't']);
        set.insert(vec!['A', 't']);

        Self { set }
    }
}

impl Default for RepeatedWords {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for RepeatedWords {
    /// A linter that checks to make sure the first word of each sentence is capitalized.
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
                        suggestions: vec![Suggestion::ReplaceWith(Vec::new())],
                        message: "Did you mean to repeat this word?".to_string(),
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

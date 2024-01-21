use hashbrown::HashSet;

use crate::{
    parsing::TokenStringExt, Dictionary, Document, Lint, LintKind, Span, Suggestion, Token,
    TokenKind,
};

/// A linter that checks to make sure the first word of each sentence is capitalized.
pub fn repeated_words(document: &Document, _dictionary: &Dictionary) -> Vec<Lint> {
    let mut lints = Vec::new();
    let set = create_match_set();

    for sentence in document.sentences() {
        let mut iter = sentence
            .iter_word_indices()
            .zip(sentence.iter_words())
            .peekable();

        while let (Some((idx_a, tok_a)), Some((idx_b, tok_b))) = (iter.next(), iter.peek()) {
            let word_a = document.get_span_content(tok_a.span);
            let word_b = document.get_span_content(tok_b.span);

            if set.contains(word_a) && word_a == word_b {
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

/// The set of words that can be considered for repetition checking.
fn create_match_set() -> HashSet<Vec<char>> {
    let mut output = HashSet::new();

    output.insert(vec!['t', 'h', 'e']);
    output.insert(vec!['T', 'h', 'e']);
    output.insert(vec!['a']);
    output.insert(vec!['A']);
    output.insert(vec!['a', 'n']);
    output.insert(vec!['A', 'n']);
    output.insert(vec!['i', 's']);
    output.insert(vec!['I', 's']);
    output.insert(vec!['w', 'i', 'l', 'l']);
    output.insert(vec!['W', 'i', 'l', 'l']);
    output.insert(vec!['l', 'i', 'k', 'e']);
    output.insert(vec!['L', 'i', 'k', 'e']);
    output.insert(vec!['t', 'h', 'a', 't']);
    output.insert(vec!['T', 'h', 'a', 't']);
    output.insert(vec!['w', 'h', 'a', 't']);
    output.insert(vec!['W', 'h', 'a', 't']);
    output.insert(vec!['w', 'h', 'i', 'c', 'h']);
    output.insert(vec!['W', 'h', 'i', 'c', 'h']);
    output.insert(vec!['b', 'e']);
    output.insert(vec!['B', 'e']);
    output.insert(vec!['a', 'n', 'd']);
    output.insert(vec!['A', 'n', 'd']);
    output.insert(vec!['I']);
    output.insert(vec!['a', 't']);
    output.insert(vec!['A', 't']);

    output
}

#[cfg(test)]
mod tests {
    use super::repeated_words;
    use crate::{Dictionary, Document};

    #[test]
    fn catches_basic() {
        let dictionary = Dictionary::new();
        let test = Document::new("I wanted the the banana.", false);
        let lints = repeated_words(&test, dictionary);
        assert!(lints.len() == 1);
    }
}

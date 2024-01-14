use crate::{document::Document, suggest_correct_spelling, Dictionary, Lint, LintKind};

use super::lint::Suggestion;

pub fn spell_check(document: &Document) -> Vec<Lint> {
    let mut lints = Vec::new();

    let dictionary = Dictionary::new();

    for word in document.words() {
        let word_chars = document.get_span_content(word.span);
        if dictionary.contains_word(word_chars) {
            continue;
        }

        let possibilities = suggest_correct_spelling(word_chars, 3, 3, &dictionary);

        let suggestions = possibilities
            .into_iter()
            .map(|word| Suggestion::ReplaceWith(word.to_vec()));

        lints.push(Lint {
            span: word.span,
            lint_kind: LintKind::Spelling,
            suggestions: suggestions.collect(),
        })
    }

    lints
}

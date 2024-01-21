use crate::{parsing::TokenStringExt, Dictionary, Document, Lint, LintKind, Span};

/// Detect and warn that the sentence is too long.
pub fn long_sentences(document: &Document, _dictionary: &Dictionary) -> Vec<Lint> {
    let mut output = Vec::new();

    for sentence in document.sentences() {
        let word_count = sentence.iter_words().count();

        if word_count > 40 {
            output.push(Lint {
                span: Span::new(sentence[0].span.start, sentence.last().unwrap().span.end),
                lint_kind: LintKind::Readability,
                message: format!("This sentence is {} words long.", word_count),
                ..Default::default()
            })
        }
    }

    output
}

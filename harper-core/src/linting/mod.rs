mod lint;
mod repeated_words;
mod sentence_capitalization;
mod spell_check;
mod unclosed_quotes;
mod wrong_quotes;

pub use lint::{Lint, LintKind, Suggestion};

use crate::{Dictionary, Document};

use self::lint::Linter;

pub fn all_linters(document: &Document, dictionary: &Dictionary) -> Vec<Lint> {
    let mut lints = Vec::new();

    let linters: [Linter; 5] = [
        spell_check::spell_check,
        sentence_capitalization::sentence_capitalization_lint,
        unclosed_quotes::unclosed_quotes,
        wrong_quotes::wrong_quotes,
        repeated_words::repeated_words_lint,
    ];

    for linter in linters {
        lints.append(&mut linter(document, dictionary));
    }

    lints.sort_by_key(|lint| lint.span.start);

    lints
}

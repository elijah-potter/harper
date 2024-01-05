mod lint;
mod sentence_capitalization;
mod spell_check;

pub use lint::{Lint, LintKind, Suggestion};

use crate::{document, Document};

use self::lint::Linter;

pub fn all_linters(document: &Document) -> Vec<Lint> {
    let mut lints = Vec::new();

    let linters: [Linter; 2] = [
        spell_check::spell_check,
        sentence_capitalization::sentence_capitalization_lint,
    ];

    for linter in linters {
        lints.append(&mut linter(document));
    }

    lints
}

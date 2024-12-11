use super::{Lint, LintSeverity, Linter, Suggestion};
use crate::document::Document;
use crate::{Token, TokenStringExt};

#[derive(Debug, Clone, Copy, Default)]
pub struct WrongQuotes;

impl Linter for WrongQuotes {
    fn lint(&mut self, document: &Document, severity: Option<LintSeverity>) -> Vec<Lint> {
        document
            .iter_quote_indices()
            .zip(document.iter_quotes())
            .filter_map(|(quote_idx, quote_token)| {
                lint_quote(document, quote_idx, quote_token, severity)
            })
            .collect()
    }
}

fn lint_quote(
    document: &Document,
    quote_idx: usize,
    quote_token: Token,
    severity: Option<LintSeverity>,
) -> Option<Lint> {
    let quote = quote_token.kind.as_quote().unwrap();

    let twin_loc = quote.twin_loc?;
    let is_left = twin_loc > quote_idx;

    let quote_char = *document.get_span_content(quote_token.span).first()?;

    let should_be = if is_left { '“' } else { '”' };

    if quote_char != should_be {
        Some(Lint {
            span: quote_token.span,
            suggestions: vec![Suggestion::ReplaceWith(vec![should_be])],
            message: "Use the better-formatted quote character.".to_string(),
            severity,
            ..Default::default()
        })
    } else {
        None
    }
}

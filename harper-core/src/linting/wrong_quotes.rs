use crate::{
    document::Document, Dictionary, Lint, LintKind, Punctuation, Suggestion, Token, TokenKind,
};

pub fn wrong_quotes(document: &Document, _dictionary: &Dictionary) -> Vec<Lint> {
    document
        .iter_quote_indices()
        .zip(document.iter_quotes())
        .filter_map(|(quote_idx, quote_token)| lint_quote(document, quote_idx, quote_token))
        .collect()
}

fn lint_quote(document: &Document, quote_idx: usize, quote_token: Token) -> Option<Lint> {
    let quote = quote_token.kind.as_quote().unwrap();

    let twin_loc = quote.twin_loc?;
    let is_left = twin_loc > quote_idx;

    let quote_char = *document.get_span_content(quote_token.span).first()?;

    let should_be = if is_left { '“' } else { '”' };

    if quote_char != should_be {
        Some(Lint {
            span: quote_token.span,
            lint_kind: LintKind::WrongQuotes,
            suggestions: vec![Suggestion::ReplaceWith(vec![should_be])],
            message: "Use the better-formatted quote character.".to_string(),
        })
    } else {
        None
    }
}

use cached::proc_macro::cached;
use harper_core::{Dictionary, Document, Lint, LintSet, Span, Suggestion};
use std::collections::HashMap;
use std::fs::read;
use tower_lsp::jsonrpc::{ErrorCode, Result};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, Diagnostic, Position, Range, TextEdit, Url, WorkspaceEdit,
};

pub fn generate_diagnostics(file_url: &Url) -> Result<Vec<Diagnostic>> {
    let file_str = open_url(file_url)?;
    let source_chars: Vec<_> = file_str.chars().collect();
    let lints = lint_string(file_str);

    let diagnostics = lints
        .into_iter()
        .map(|lint| lint_to_diagnostic(lint, &source_chars))
        .collect();

    Ok(diagnostics)
}

pub fn generate_code_actions(url: &Url, range: Range) -> Result<Vec<CodeAction>> {
    let file_str = open_url(url)?;
    let source_chars: Vec<_> = file_str.chars().collect();
    let lints = lint_string(file_str);

    // Find lints whose span overlaps with range
    let span = range_to_span(&source_chars, range);

    let actions = lints
        .into_iter()
        .filter(|lint| lint.span.overlaps_with(span))
        .flat_map(|lint| lint_to_code_actions(&lint, url, &source_chars).collect::<Vec<_>>())
        .collect();

    Ok(actions)
}

fn lint_to_code_actions<'a>(
    lint: &'a Lint,
    url: &'a Url,
    source: &'a [char],
) -> impl Iterator<Item = CodeAction> + 'a {
    lint.suggestions.iter().flat_map(|suggestion| {
        let range = span_to_range(source, lint.span);

        let Suggestion::ReplaceWith(with) = suggestion;
        Some(CodeAction {
            title: suggestion.to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: Some(WorkspaceEdit {
                changes: Some(HashMap::from([(
                    url.clone(),
                    vec![TextEdit {
                        range,

                        new_text: with.iter().collect(),
                    }],
                )])),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        })
    })
}

fn open_url(url: &Url) -> Result<String> {
    let file = read(url.path())
        .map_err(|_err| tower_lsp::jsonrpc::Error::new(ErrorCode::InternalError))?;
    Ok(String::from_utf8(file).unwrap())
}

#[cached]
fn lint_string(text: String) -> Vec<Lint> {
    let document = Document::new(&text, true);
    let dictionary = Dictionary::new();
    document.run_lint_set(&LintSet::default(), dictionary)
}

fn lint_to_diagnostic(lint: Lint, source: &[char]) -> Diagnostic {
    let range = span_to_range(source, lint.span);

    Diagnostic {
        range,
        severity: None,
        code: None,
        code_description: None,
        source: Some("Harper".to_string()),
        message: lint.message,
        related_information: None,
        tags: None,
        data: None,
    }
}

fn span_to_range(source: &[char], span: Span) -> Range {
    let start = index_to_position(source, span.start);
    let end = index_to_position(source, span.end);

    Range { start, end }
}

fn index_to_position(source: &[char], index: usize) -> Position {
    let before = &source[0..index];
    let newline_indices: Vec<_> = before
        .iter()
        .enumerate()
        .filter_map(|(idx, c)| if *c == '\n' { Some(idx) } else { None })
        .collect();

    let lines = newline_indices.len();
    let cols = index - newline_indices.last().copied().unwrap_or(1) - 1;

    Position {
        line: lines as u32,
        character: cols as u32,
    }
}

fn position_to_index(source: &[char], position: Position) -> usize {
    let newline_indices =
        source
            .iter()
            .enumerate()
            .filter_map(|(idx, c)| if *c == '\n' { Some(idx) } else { None });

    let line_start_idx = newline_indices
        .take(position.line as usize)
        .last()
        .unwrap_or(0);

    line_start_idx + position.character as usize + 1
}

fn range_to_span(source: &[char], range: Range) -> Span {
    let start = position_to_index(source, range.start);
    let end = position_to_index(source, range.end);

    Span::new(start, end)
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Position;

    use super::{index_to_position, position_to_index};

    #[test]
    fn reversible_position_conv() {
        let source: Vec<_> = "There was a man,\n his voice had timbre,\n unlike a boy."
            .chars()
            .collect();

        let a = Position {
            line: 2,
            character: 3,
        };

        let b = position_to_index(&source, a);

        assert_eq!(b, 43);

        let c = index_to_position(&source, b);

        let d = position_to_index(&source, a);

        assert_eq!(a, c);
        assert_eq!(b, d);
    }
}

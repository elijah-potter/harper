use harper_core::{all_linters, Dictionary, Document, Lint, Span};
use lsp_types::{Diagnostic, Position, Range, Url};
use std::fs::read;

pub fn generate_diagnostics(file_uri: Url) -> anyhow::Result<Vec<Diagnostic>> {
    let file = read(file_uri.path())?;
    let file_str = String::from_utf8(file)?;

    let document = Document::new(&file_str);
    let dictionary = Dictionary::new();
    let lints = all_linters(&document, dictionary);

    let source_chars: Vec<_> = file_str.chars().collect();

    let diagnostics = lints
        .into_iter()
        .map(|lint| lint_to_diagnostic(lint, &source_chars))
        .collect();

    Ok(diagnostics)
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
    let cols = index - newline_indices.last().copied().unwrap_or(0);

    Position {
        line: lines as u32,
        character: cols as u32,
    }
}

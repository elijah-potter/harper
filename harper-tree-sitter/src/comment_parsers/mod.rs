mod go;
mod unit;

pub use go::Go;
use harper_core::Span;
pub use unit::Unit;

/// Get the span of a tree-sitter-produced comment that doesn't include the
/// comment openers and closers.
fn without_intiators(source: &[char]) -> Span {
    // Skip over the comment start characters
    let actual_start = source
        .iter()
        .position(|c| !is_comment_character(*c))
        .unwrap_or(0);

    // Chop off the end
    let actual_end = source.len()
        - source
            .iter()
            .rev()
            .position(|c| !is_comment_character(*c))
            .unwrap_or(0);

    Span::new(actual_start, actual_end)
}

fn is_comment_character(c: char) -> bool {
    matches!(c, '#' | '-' | '/' | '*')
}

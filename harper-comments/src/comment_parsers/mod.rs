mod go;
mod javadoc;
mod jsdoc;
mod unit;

pub use go::Go;
use harper_core::Span;
pub use javadoc::JavaDoc;
pub use jsdoc::JsDoc;
pub use unit::Unit;

/// Get the span of a tree-sitter-produced comment that doesn't include the
/// comment openers and closers.
fn without_initiators(source: &[char]) -> Span {
    // Skip over the comment start characters
    let actual_start = source
        .iter()
        .position(|c| !is_comment_character(*c) && !c.is_whitespace())
        .unwrap_or(source.len());

    // Chop off the end
    let actual_end = source.len()
        - source
            .iter()
            .rev()
            .position(|c| !is_comment_character(*c) && !c.is_whitespace())
            .unwrap_or(0);

    Span::new(actual_start, actual_end)
}

fn is_comment_character(c: char) -> bool {
    matches!(c, '#' | '-' | '/' | '*' | '!')
}

#[cfg(test)]
mod tests {
    use super::without_initiators;

    #[test]
    fn cleans_empty_comment() {
        let source: Vec<_> = "///".chars().collect();
        assert_eq!(without_initiators(&source).len(), 0);
    }

    #[test]
    fn cleans_empty_comment_with_whitespace() {
        let source: Vec<_> = "///   ".chars().collect();
        assert_eq!(without_initiators(&source).len(), 0);
    }
}

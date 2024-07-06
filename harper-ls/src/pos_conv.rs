use harper_core::Span;
use tower_lsp::lsp_types::{Position, Range};

/// This module includes various conversions from the index-based [`Span`]s that
/// Harper uses, and the Ranges that the LSP uses.

pub fn span_to_range(source: &[char], span: Span) -> Range {
    let start = index_to_position(source, span.start);
    let end = index_to_position(source, span.end);

    Range { start, end }
}

fn index_to_position(source: &[char], index: usize) -> Position {
    let before = &source[0..index];
    let newline_indices: Vec<_> = before
        .iter()
        .enumerate()
        .filter_map(|(idx, c)| if *c == '\n' { Some(idx + 1) } else { None })
        .collect();

    let lines = newline_indices.len();

    let last_newline_idx = newline_indices.last().copied().unwrap_or(0);

    let cols: usize = source[last_newline_idx..index]
        .iter()
        .map(|c| c.len_utf16())
        .sum();

    Position {
        line: lines as u32,
        character: cols as u32
    }
}

fn position_to_index(source: &[char], position: Position) -> usize {
    let newline_indices =
        source
            .iter()
            .enumerate()
            .filter_map(|(idx, c)| if *c == '\n' { Some(idx + 1) } else { None });

    let line_start_idx = newline_indices
        .take(position.line as usize)
        .last()
        .unwrap_or(0);

    let mut traversed_cols = 0;

    for (traversed_chars, c) in source[line_start_idx..].iter().enumerate() {
        if traversed_cols == position.character as usize {
            return line_start_idx + traversed_chars;
        }

        traversed_cols += c.len_utf16();
    }

    line_start_idx
}

pub fn range_to_span(source: &[char], range: Range) -> Span {
    let start = position_to_index(source, range.start);
    let end = position_to_index(source, range.end);

    Span::new(start, end)
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Position;

    use super::{index_to_position, position_to_index};

    #[test]
    fn first_line_correct() {
        let source: Vec<_> = "Hello there.".chars().collect();

        let start = Position {
            line: 0,
            character: 4
        };

        let i = position_to_index(&source, start);

        assert_eq!(i, 4);

        let p = index_to_position(&source, i);

        assert_eq!(p, start)
    }

    #[test]
    fn reversible_position_conv() {
        let source: Vec<_> = "There was a man,\n his voice had timbre,\n unlike a boy."
            .chars()
            .collect();

        let a = Position {
            line: 1,
            character: 2
        };

        let b = position_to_index(&source, a);

        assert_eq!(b, 19);

        let c = index_to_position(&source, b);

        let d = position_to_index(&source, a);

        assert_eq!(a, c);
        assert_eq!(b, d);
    }
}

use std::iter::repeat;

use crate::{line_col::LineCol, span::Span};

pub struct Lint {
    span: Span,
    lint_kind: LintKind,
}

impl Lint {
    pub fn new(span: Span) -> Self {
        Self {
            span,
            lint_kind: LintKind::MissingWhiteSpace,
        }
    }

    pub fn display(&self, source: &[char]) -> String {
        let mut rect = to_rect(source);

        let start_coord = LineCol::from_index(self.span.start, source);
        crop(start_coord, LineCol::new(3, 3), &mut rect);
        outline_text(&mut rect);

        rect_to_string(&mut rect)
    }
}

pub enum LintKind {
    MissingWhiteSpace,
}

/// Converts a flat (1 dimensional) array of characters to a plane (2 dimensional) format.
fn to_rect<'a>(source: impl IntoIterator<Item = &'a char>) -> Vec<Vec<char>> {
    let mut output = Vec::new();
    let mut current_line = Vec::new();

    for c in source {
        if *c == '\n' {
            output.push(current_line.clone());
            current_line.clear()
        } else {
            current_line.push(*c);
        }
    }

    output
}

/// Add spaces on all lines whose length is less than the longest line until all are equal length.
fn pad_lines(source: &mut Vec<Vec<char>>) {
    let longest_line = source.iter().map(|line| line.len()).max().unwrap_or(0);

    for line in source {
        line.extend(repeat(' ').take(longest_line - line.len()));
    }
}

/// Wrap text in visual box
fn outline_text(source: &mut Vec<Vec<char>>) {
    pad_lines(source);

    source.insert(0, vec!['─'; source[0].len()]);
    source.iter_mut().for_each(|line| {
        line.insert(0, '┃');
        line.push('┃');
    });
    source.push(vec!['─'; source[0].len()])
}

fn rect_to_string(source: &mut Vec<Vec<char>>) -> String {
    let mut output = String::new();

    for line in source {
        output.extend(line.iter());
        output.push('\n');
    }

    output
}

fn crop(top_left: LineCol, size: LineCol, source: &mut Vec<Vec<char>>) {
    source.drain(0..top_left.line);
    source.drain(top_left.line + size.line..);

    source.retain(|item| item.len() > top_left.col);

    for item in source.iter_mut() {
        if item.len() > top_left.col {
            item.drain(0..top_left.col);

            let end = top_left.col + size.col;
            if item.len() > end {
                item.drain(end..);
            }
        }
    }
}

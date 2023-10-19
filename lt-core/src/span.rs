use crate::line_col::LineCol;

/// A window in an [char].
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn get_content<'a>(&self, source: &'a [char]) -> &'a [char] {
        &source[self.start..self.end]
    }
}

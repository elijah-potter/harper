use serde::{Deserialize, Serialize};

/// A window in a [char].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

    pub fn get_content_string(&self, source: &[char]) -> String {
        String::from_iter(self.get_content(source))
    }
}

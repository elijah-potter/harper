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

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn get_content<'a>(&self, source: &'a [char]) -> &'a [char] {
        if cfg!(debug_assertions) {
            assert!(self.start < self.end);
            assert!(self.end <= source.len());
        }

        &source[self.start..self.end]
    }

    pub fn get_content_string(&self, source: &[char]) -> String {
        String::from_iter(self.get_content(source))
    }

    pub fn set_len(&mut self, length: usize) {
        self.end = self.start + length;
    }

    pub fn with_len(&self, length: usize) -> Self {
        let mut cloned = *self;
        cloned.set_len(length);
        cloned
    }
}

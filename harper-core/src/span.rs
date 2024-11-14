use std::ops::Range;

use serde::{Deserialize, Serialize};

/// A window in a [`char`] sequence.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        if start > end {
            panic!("{} > {}", start, end);
        }
        Self { start, end }
    }

    pub fn new_with_len(start: usize, len: usize) -> Self {
        Self {
            start,
            end: start + len,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, idx: usize) -> bool {
        assert!(self.start <= self.end);

        self.start <= idx && idx < self.end
    }

    pub fn overlaps_with(&self, other: Self) -> bool {
        (self.start < other.end) && (other.start < self.end)
    }

    /// Get the associated content. Will return [`None`] if any aspect is
    /// invalid.
    pub fn try_get_content<'a>(&self, source: &'a [char]) -> Option<&'a [char]> {
        if (self.start > self.end) || (self.start >= source.len()) || (self.end > source.len()) {
            if self.is_empty() {
                return Some(&source[0..0]);
            }
            return None;
        }

        Some(&source[self.start..self.end])
    }

    /// Get the associated content. Will panic if any aspect is invalid.
    pub fn get_content<'a>(&self, source: &'a [char]) -> &'a [char] {
        self.try_get_content(source).unwrap()
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

    // Add an amount to both [`Self::start`] and [`Self::end`]
    pub fn push_by(&mut self, by: usize) {
        self.start += by;
        self.end += by;
    }

    // Subtract an amount to both [`Self::start`] and [`Self::end`]
    pub fn pull_by(&mut self, by: usize) {
        self.start -= by;
        self.end -= by;
    }

    // Add an amount to a copy of both [`Self::start`] and [`Self::end`]
    pub fn pushed_by(&self, by: usize) -> Self {
        let mut clone = *self;
        clone.start += by;
        clone.end += by;
        clone
    }

    // Subtract an amount to a copy of both [`Self::start`] and [`Self::end`]
    pub fn pulled_by(&self, by: usize) -> Self {
        let mut clone = *self;
        clone.start -= by;
        clone.end -= by;
        clone
    }

    // Add an amount a copy of both [`Self::start`] and [`Self::end`]
    pub fn with_offset(&self, by: usize) -> Self {
        let mut clone = *self;
        clone.push_by(by);
        clone
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self::new(value.start, value.end)
    }
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        value.start..value.end
    }
}

#[cfg(test)]
mod tests {
    use crate::Span;

    #[test]
    fn overlaps() {
        assert!(Span::new(0, 5).overlaps_with(Span::new(3, 6)));
        assert!(Span::new(0, 5).overlaps_with(Span::new(2, 3)));
        assert!(Span::new(0, 5).overlaps_with(Span::new(4, 5)));
        assert!(Span::new(0, 5).overlaps_with(Span::new(4, 4)));

        assert!(!Span::new(0, 3).overlaps_with(Span::new(3, 5)));
    }
}

use crate::Span;

/// A Masker is a tool that can be composed to eliminate chunks of text from
/// being parsed. They can be composed to do things like isolate comments from a
/// programming language or disable linting for languages that have been
/// determined to not be English.
///
/// This is primarily used by [`crate::parsers::Mask`] to create parsers for
/// things like comments of programming languages.
pub trait Masker: Send + Sync {
    fn create_mask(&mut self, source: &[char]) -> Mask;
}

/// Identifies portions of a [`char`] sequence that should __not__ be ignored by
/// Harper.
pub struct Mask {
    // Right now, there aren't any use-cases where we can't treat this as a stack.
    //
    // Assumed that no elements overlap and exist in sorted order.
    pub(self) allowed: Vec<Span>,
}

impl Mask {
    /// Create a new Mask for a given piece of text, marking all text as
    /// disallowed.
    pub fn new_blank() -> Self {
        Self {
            allowed: Vec::new(),
        }
    }

    pub fn iter_allowed<'a>(
        &'a self,
        source: &'a [char],
    ) -> impl Iterator<Item = (Span, &'a [char])> + '_ {
        self.allowed.iter().map(|s| (*s, s.get_content(source)))
    }

    /// Mark a span of the text as allowed.
    pub fn push_allowed(&mut self, allowed: Span) {
        if let Some(last) = self.allowed.last_mut() {
            assert!(allowed.start >= last.end);

            if allowed.start == last.end {
                last.end = allowed.end;
                return;
            }
        }

        self.allowed.push(allowed)
    }

    /// Merge chunks that are only separated by whitespace.
    pub fn merge_whitespace_sep(&mut self, source: &[char]) {
        let mut after = Vec::with_capacity(self.allowed.len());

        let mut iter = 0..self.allowed.len();

        while let Some(i) = iter.next() {
            let a = self.allowed[i];

            if let Some(b) = self.allowed.get(i + 1) {
                let sep = Span::new(a.end, b.start);
                let sep_content = sep.get_content(source);

                if sep_content.iter().all(|c| c.is_whitespace() || *c == '\n') {
                    iter.next();
                    after.push(Span::new(a.start, b.end));
                    continue;
                }
            }

            after.push(a);
        }

        if self.allowed.len() != after.len() {
            self.allowed = after;
            self.merge_whitespace_sep(source);
        } else {
            self.allowed = after;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Mask, Span};

    #[test]
    fn bumps_existing() {
        let mut mask = Mask::new_blank();

        mask.push_allowed(Span::new_with_len(0, 1));
        mask.push_allowed(Span::new_with_len(1, 2));

        assert_eq!(mask.allowed.len(), 1)
    }

    #[test]
    fn merges_whitespace_sep() {
        let source: Vec<_> = "word word\nword".chars().collect();

        let mut mask = Mask::new_blank();
        mask.push_allowed(Span::new_with_len(0, 4));
        mask.push_allowed(Span::new_with_len(5, 4));
        mask.push_allowed(Span::new_with_len(10, 4));

        assert_eq!(mask.allowed.len(), 3);

        mask.merge_whitespace_sep(&source);

        assert_eq!(mask.allowed.len(), 1);
    }
}

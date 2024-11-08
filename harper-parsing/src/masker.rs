use crate::Mask;

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

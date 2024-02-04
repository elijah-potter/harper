use std::sync::Arc;

use super::dictionary::Dictionary;

/// A simple wrapper over [`Dictionary`] that allows
/// one to merge multiple dictionaries without copying.
#[derive(Clone)]
pub struct MergedDictionary<T>
where
    T: Dictionary + Clone,
{
    children: Vec<Arc<T>>,
}

impl<T> MergedDictionary<T>
where
    T: Dictionary + Clone,
{
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn add_dictionary(&mut self, dictionary: Arc<T>) {
        self.children.push(dictionary.clone());
    }
}

impl<T> Default for MergedDictionary<T>
where
    T: Dictionary + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Dictionary for MergedDictionary<T>
where
    T: Dictionary + Clone,
{
    fn contains_word(&self, word: &[char]) -> bool {
        for child in &self.children {
            if child.contains_word(word) {
                return true;
            }
        }
        false
    }

    fn words_iter(&self) -> impl Iterator<Item = &'_ [char]> {
        self.children.iter().flat_map(|c| c.words_iter())
    }

    fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + '_> {
        Box::new(
            self.children
                .iter()
                .flat_map(move |c| c.words_with_len_iter(len)),
        )
    }
}

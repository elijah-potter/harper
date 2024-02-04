use blanket::blanket;

#[blanket(derive(Arc))]
pub trait Dictionary: Send + Sync + Clone {
    fn contains_word(&self, word: &[char]) -> bool;
    fn words_iter(&self) -> impl Iterator<Item = &'_ [char]>;
    /// Iterate over all the words in the dictionary of a given length
    fn words_with_len_iter(&self, len: usize) -> Box<dyn Iterator<Item = &'_ [char]> + '_>;
}

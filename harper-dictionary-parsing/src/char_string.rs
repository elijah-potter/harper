use smallvec::SmallVec;

/// A char sequence that improves cache locality.
/// Most English words are fewer than 12 characters.
pub type CharString = SmallVec<[char; 12]>;

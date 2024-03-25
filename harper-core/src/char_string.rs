use smallvec::SmallVec;

pub type CharString = SmallVec<[char; 6]>;

pub trait CharStringExt {
    fn to_lower(&self) -> CharString;
}

impl CharStringExt for [char] {
    fn to_lower(&self) -> CharString {
        let mut out = CharString::with_capacity(self.len());

        out.extend(self.iter().flat_map(|v| v.to_lowercase()));

        out
    }
}

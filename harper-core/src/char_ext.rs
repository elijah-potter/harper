use unicode_width::UnicodeWidthChar;

use crate::Punctuation;

pub trait CharExt {
    /// Whether a character can be a component of a word.
    fn is_lingual(&self) -> bool;
    fn is_emoji(&self) -> bool;
    fn is_punctuation(&self) -> bool;
}

impl CharExt for char {
    fn is_lingual(&self) -> bool {
        !self.is_whitespace()
            && !self.is_numeric()
            && !self.is_emoji()
            && matches!(self.width(), Some(1..))
            && !self.is_punctuation()
            && (self.is_alphabetic() || matches!(self, '\u{00C0}'..='\u{00FF}'))
    }

    fn is_emoji(&self) -> bool {
        let Some(block) = unicode_blocks::find_unicode_block(*self) else {
            return false;
        };

        let blocks = [
            unicode_blocks::SPECIALS,
            unicode_blocks::EMOTICONS,
            unicode_blocks::MISCELLANEOUS_SYMBOLS,
            unicode_blocks::VARIATION_SELECTORS,
            unicode_blocks::SUPPLEMENTAL_SYMBOLS_AND_PICTOGRAPHS
        ];

        blocks.contains(&block)
    }

    fn is_punctuation(&self) -> bool {
        Punctuation::from_char(*self).is_some()
    }
}

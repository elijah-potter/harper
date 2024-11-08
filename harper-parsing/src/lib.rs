mod lexing;
mod markdown;
mod mask;
mod mask_parser;
mod masker;
mod plain_english;

use blanket::blanket;
use harper_data::Token;
pub use markdown::Markdown;
pub use mask::Mask;
pub use mask_parser::MaskParser;
pub use masker::Masker;
pub use plain_english::PlainEnglish;

#[cfg(not(feature = "concurrent"))]
#[blanket(derive(Box))]
pub trait Parser {
    fn parse(&mut self, source: &[char]) -> Vec<Token>;
}

#[cfg(feature = "concurrent")]
#[blanket(derive(Box))]
pub trait Parser: Send + Sync {
    fn parse(&mut self, source: &[char]) -> Vec<Token>;
}

pub trait StrParser {
    fn parse_str(&mut self, source: impl AsRef<str>) -> Vec<Token>;
}

impl<T> StrParser for T
where
    T: Parser,
{
    fn parse_str(&mut self, source: impl AsRef<str>) -> Vec<Token> {
        let source: Vec<_> = source.as_ref().chars().collect();
        self.parse(&source)
    }
}

#[cfg(test)]
mod tests {
    use harper_data::Punctuation;
    use harper_data::TokenKind::{self, *};

    use super::{Markdown, Parser, PlainEnglish};

    fn assert_tokens_eq(
        test_str: impl AsRef<str>,
        expected: &[TokenKind],
        parser: &mut impl Parser,
    ) {
        let chars: Vec<_> = test_str.as_ref().chars().collect();
        let tokens = parser.parse(&chars);
        let kinds: Vec<_> = tokens.into_iter().map(|v| v.kind).collect();

        assert_eq!(&kinds, expected)
    }

    fn assert_tokens_eq_plain(test_str: impl AsRef<str>, expected: &[TokenKind]) {
        let mut parser = PlainEnglish;
        assert_tokens_eq(test_str, expected, &mut parser);
    }

    fn assert_tokens_eq_md(test_str: impl AsRef<str>, expected: &[TokenKind]) {
        let mut parser = Markdown;

        assert_tokens_eq(test_str, expected, &mut parser)
    }

    #[test]
    fn single_letter() {
        assert_tokens_eq_plain("a", &[TokenKind::blank_word()])
    }

    #[test]
    fn sentence() {
        assert_tokens_eq_plain(
            "hello world, my friend",
            &[
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
                Punctuation(Punctuation::Comma),
                Space(1),
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
            ],
        )
    }

    #[test]
    fn sentence_md() {
        assert_tokens_eq_md(
            "__hello__ world, [my]() friend",
            &[
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
                Punctuation(Punctuation::Comma),
                Space(1),
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
            ],
        );
    }

    #[test]
    fn inserts_newlines() {
        assert_tokens_eq_md(
            "__hello__ world,\n\n[my]() friend",
            &[
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
                Punctuation(Punctuation::Comma),
                Newline(2),
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
            ],
        );
    }

    /// Make sure that the English parser correctly identifies non-English
    /// characters as part of the same word.
    #[test]
    fn parses_non_english() {
        assert_tokens_eq_plain("Løvetann", &[TokenKind::blank_word()]);
        assert_tokens_eq_plain("Naïve", &[TokenKind::blank_word()]);
    }
}

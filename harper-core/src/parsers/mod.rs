mod markdown;
mod plain_english;

pub use markdown::Markdown;
pub use plain_english::PlainEnglish;

pub use crate::token::{Token, TokenKind, TokenStringExt};

pub trait Parser: Send + Sync {
    fn parse(&mut self, source: &[char]) -> Vec<Token>;
}

pub trait StrParser {
    fn parse_str(&mut self, source: impl AsRef<str>) -> Vec<Token>;
}

impl<T> StrParser for T
where
    T: Parser
{
    fn parse_str(&mut self, source: impl AsRef<str>) -> Vec<Token> {
        let source: Vec<_> = source.as_ref().chars().collect();
        self.parse(&source)
    }
}

#[cfg(test)]
mod tests {
    use super::{Markdown, Parser, PlainEnglish};
    use crate::Punctuation;
    use crate::TokenKind::{self, *};

    fn assert_tokens_eq(
        test_str: impl AsRef<str>,
        expected: &[TokenKind],
        parser: &mut impl Parser
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
        assert_tokens_eq_plain("a", &[Word])
    }

    #[test]
    fn sentence() {
        assert_tokens_eq_plain(
            "hello world, my friend",
            &[
                Word,
                Space(1),
                Word,
                Punctuation(Punctuation::Comma),
                Space(1),
                Word,
                Space(1),
                Word
            ]
        )
    }

    #[test]
    fn sentence_md() {
        assert_tokens_eq_md(
            "__hello__ world, [my]() friend",
            &[
                Word,
                Space(1),
                Word,
                Punctuation(Punctuation::Comma),
                Space(1),
                Word,
                Space(1),
                Word,
                Newline(2)
            ]
        );
    }

    #[test]
    fn inserts_newlines() {
        assert_tokens_eq_md(
            "__hello__ world,\n\n[my]() friend",
            &[
                Word,
                Space(1),
                Word,
                Punctuation(Punctuation::Comma),
                Newline(2),
                Word,
                Space(1),
                Word,
                Newline(2)
            ]
        );
    }

    /// Make sure that the English parser correctly identifies non-english
    /// characters as part of the same word.
    #[test]
    fn parses_non_english() {
        assert_tokens_eq_plain("Løvetann", &[Word]);
        assert_tokens_eq_plain("Naïve", &[Word]);
    }
}

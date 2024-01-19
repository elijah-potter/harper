use std::fmt::Display;

use itertools::Itertools;

use crate::{
    lex_to_end,
    linting::Suggestion,
    span::Span,
    FatToken,
    Punctuation::{self},
    Token, TokenKind,
};

pub struct Document {
    source: Vec<char>,
    tokens: Vec<Token>,
}

impl Document {
    // Lexes and parses text to produce a document.
    pub fn new(text: &str) -> Self {
        let source: Vec<_> = text.chars().collect();
        let tokens = lex_to_end(&source);

        let mut doc = Self { source, tokens };
        doc.parse();

        doc
    }

    /// Re-parse important language constructs.
    ///
    /// Should be run after every change to the underlying [`Self::source`].
    fn parse(&mut self) {
        self.match_quotes();
    }

    pub fn iter_quote_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.tokens.iter().enumerate().filter_map(|(idx, token)| {
            if let TokenKind::Punctuation(Punctuation::Quote(_)) = &token.kind {
                Some(idx)
            } else {
                None
            }
        })
    }

    pub fn iter_quotes(&self) -> impl Iterator<Item = Token> + '_ {
        self.iter_quote_indices().map(|idx| self.tokens[idx])
    }

    /// Searches for quotation marks and fills the [`Punctuation::Quote::twin_loc`] field.
    /// This is on a best effort basis.
    ///
    /// Current algorithm is very basic and could use some work.
    fn match_quotes(&mut self) {
        let quote_indices: Vec<usize> = self.iter_quote_indices().collect();

        for i in 0..quote_indices.len() / 2 {
            let a_i = quote_indices[i * 2];
            let b_i = quote_indices[i * 2 + 1];

            {
                let a = self.tokens[a_i].kind.as_mut_quote().unwrap();
                a.twin_loc = Some(b_i);
            }

            {
                let b = self.tokens[b_i].kind.as_mut_quote().unwrap();
                b.twin_loc = Some(a_i);
            }
        }
    }

    pub fn tokens(&self) -> impl Iterator<Item = Token> + '_ {
        self.tokens.iter().copied()
    }

    pub fn fat_tokens(&self) -> impl Iterator<Item = FatToken> + '_ {
        self.tokens().map(|token| token.to_fat(&self.source))
    }

    /// Iterate over the locations of the sentence terminators in the document.
    fn sentence_terminators(&self) -> impl Iterator<Item = usize> + '_ {
        self.tokens.iter().enumerate().filter_map(|(index, token)| {
            if let Token {
                kind: TokenKind::Punctuation(punct),
                ..
            } = token
            {
                if is_sentence_terminator(punct) {
                    Some(index)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    pub fn sentences(&self) -> impl Iterator<Item = &'_ [Token]> + '_ {
        let first_sentence = self
            .sentence_terminators()
            .next()
            .map(|first_term| &self.tokens[0..=first_term]);

        let rest = self
            .sentence_terminators()
            .tuple_windows()
            .map(move |(a, b)| &self.tokens[a + 1..=b]);

        first_sentence.into_iter().chain(rest)
    }

    /** Returns all tokens whose `kind` is [`Punctuation::Word`] */
    pub fn words(&self) -> impl Iterator<Item = Token> + '_ {
        self.tokens
            .iter()
            .filter(|token| token.kind.is_word())
            .cloned()
    }

    pub fn get_span_content(&self, span: Span) -> &[char] {
        span.get_content(&self.source)
    }

    pub fn get_span_content_str(&self, span: Span) -> String {
        String::from_iter(self.get_span_content(span))
    }

    pub fn get_full_string(&self) -> String {
        self.get_span_content_str(Span {
            start: 0,
            end: self.source.len(),
        })
    }

    pub fn apply_suggestion(&mut self, suggestion: &Suggestion, span: Span) {
        match suggestion {
            Suggestion::ReplaceWith(chars) => {
                // Avoid allocation if possible
                if chars.len() == span.len() {
                    for (index, c) in chars.iter().enumerate() {
                        self.source[index + span.start] = *c
                    }
                } else {
                    let popped = self.source.split_off(span.start);

                    self.source.extend(chars);
                    self.source.extend(popped.into_iter().skip(span.len()));
                }
            }
        }

        self.parse();
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for token in &self.tokens {
            write!(f, "{}", self.get_span_content_str(token.span))?;
        }

        Ok(())
    }
}

fn is_sentence_terminator(punctuation: &Punctuation) -> bool {
    [
        Punctuation::Period,
        Punctuation::Bang,
        Punctuation::Question,
    ]
    .contains(punctuation)
}

#[cfg(test)]
mod tests {
    use super::Document;
    use crate::Token;

    impl Document {
        fn from_raw_parts(source: Vec<char>, tokens: Vec<Token>) -> Self {
            Self { source, tokens }
        }
    }

    #[test]
    fn parses_sentances_correctly() {
        let text = "There were three little pigs. They built three little homes.";
        let document = Document::new(text);

        let mut sentence_strs = vec![];

        for sentence in document.sentences() {
            sentence_strs.push(
                Document::from_raw_parts(document.source.clone(), sentence.to_vec()).to_string(),
            );
        }

        assert_eq!(
            sentence_strs,
            vec![
                "There were three little pigs.",
                " They built three little homes."
            ]
        )
    }
}

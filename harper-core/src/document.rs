use std::fmt::Display;

use itertools::Itertools;

use crate::{
    linting::Suggestion,
    parsing::{lex_to_end, lex_to_end_md},
    span::Span,
    FatToken,
    Punctuation::{self},
    Token, TokenKind,
};

pub struct Document {
    source: Vec<char>,
    tokens: Vec<Token>,
    markdown: bool,
}

impl Document {
    // Lexes and parses text to produce a document.
    //
    // Choosing to parse with markdown may have a performance penalty
    pub fn new(text: &str, markdown: bool) -> Self {
        let source: Vec<_> = text.chars().collect();

        let mut doc = Self {
            source,
            tokens: Vec::new(),
            markdown,
        };
        doc.parse();

        doc
    }

    /// Re-parse important language constructs.
    ///
    /// Should be run after every change to the underlying [`Self::source`].
    fn parse(&mut self) {
        if self.markdown {
            self.tokens = lex_to_end_md(&self.source);
        } else {
            self.tokens = lex_to_end(&self.source);
        }

        self.condense_contractions();
        // Since quote matches depend on token indices.
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

    /// Searches for contractions and condenses them down into single tokens
    fn condense_contractions(&mut self) {
        if self.tokens.len() < 3 {
            return;
        }

        // Indices of the three token stretches we are going to condense.
        let mut replace_starts = Vec::new();

        for idx in 0..self.tokens.len() - 2 {
            let a = self.tokens[idx];
            let b = self.tokens[idx + 1];
            let c = self.tokens[idx + 2];

            if matches!(
                (a.kind, b.kind, c.kind),
                (
                    TokenKind::Word,
                    TokenKind::Punctuation(Punctuation::Apostrophe),
                    TokenKind::Word
                )
            ) {
                // Ensure there is no overlapping between replacements
                let should_replace = if let Some(last_idx) = replace_starts.last() {
                    *last_idx < idx - 2
                } else {
                    true
                };

                if should_replace {
                    replace_starts.push(idx);
                    self.tokens[idx].span.end = c.span.end;
                }
            }
        }

        // Trim
        let old = self.tokens.clone();
        self.tokens.clear();

        // Keep first chunk.
        self.tokens.extend_from_slice(
            &old[0..replace_starts
                .first()
                .copied()
                .unwrap_or(replace_starts.len())],
        );

        let mut iter = replace_starts.iter().peekable();

        while let (Some(a_idx), b) = (iter.next(), iter.peek()) {
            self.tokens.push(old[*a_idx]);

            if let Some(b_idx) = b {
                self.tokens.extend_from_slice(&old[a_idx + 3..**b_idx]);
            }
        }

        // Keep last chunk.
        self.tokens.extend_from_slice(
            &old[replace_starts
                .last()
                .map(|v| v + 3)
                .unwrap_or(replace_starts.len())..],
        )
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
                    return Some(index);
                }
            }
            None
        })
    }

    /// Get the index of the last sentence terminator.
    fn last_sentence_terminator(&self) -> Option<usize> {
        self.tokens
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, token)| {
                if let Token {
                    kind: TokenKind::Punctuation(punct),
                    ..
                } = token
                {
                    if is_sentence_terminator(punct) {
                        return Some(index);
                    }
                }
                None
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

        let last = if let Some(last_i) = self.last_sentence_terminator() {
            if last_i + 1 < self.tokens.len() {
                Some(&self.tokens[last_i + 1..])
            } else {
                None
            }
        } else {
            Some(self.tokens.as_slice())
        };

        first_sentence.into_iter().chain(rest).chain(last)
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
    use crate::{Span, Token, TokenKind};

    impl Document {
        fn from_raw_parts(source: Vec<char>, tokens: Vec<Token>, markdown: bool) -> Self {
            Self {
                source,
                tokens,
                markdown,
            }
        }
    }

    fn assert_condensed_contractions(text: &str, final_tok_count: usize) {
        let mut document = Document::new(text, false);
        dbg!(&document.tokens);
        document.condense_contractions();

        assert_eq!(document.tokens.len(), final_tok_count);

        let mut document = Document::new(text, true);
        dbg!(&document.tokens);
        document.condense_contractions();

        assert_eq!(document.tokens.len(), final_tok_count);
    }

    #[test]
    fn simple_contraction() {
        assert_condensed_contractions("isn't", 1);
    }

    #[test]
    fn simple_contraction2() {
        assert_condensed_contractions("wasn't", 1);
    }

    #[test]
    fn simple_contraction3() {
        assert_condensed_contractions("There's", 1);
    }

    #[test]
    fn medium_contraction() {
        assert_condensed_contractions("isn't wasn't", 3);
    }

    #[test]
    fn medium_contraction2() {
        assert_condensed_contractions("There's no way", 5);
    }

    #[test]
    fn parses_sentences_correctly() {
        let text = "There were three little pigs. They built three little homes.";
        let document = Document::new(text, false);

        let mut sentence_strs = vec![];

        for sentence in document.sentences() {
            sentence_strs.push(
                Document::from_raw_parts(document.source.clone(), sentence.to_vec(), false)
                    .to_string(),
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

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::Display;

use itertools::Itertools;
use paste::paste;

use crate::linting::Suggestion;
use crate::parsers::{Markdown, Parser, PlainEnglish};
use crate::punctuation::Punctuation;
use crate::span::Span;
use crate::token::NumberSuffix;
use crate::{FatToken, Lrc, Token, TokenKind, TokenStringExt};

pub struct Document {
    source: Lrc<Vec<char>>,
    tokens: Vec<Token>,
    parser: Box<dyn Parser>
}

impl Default for Document {
    fn default() -> Self {
        Self::new("", Box::new(PlainEnglish))
    }
}

impl Document {
    /// Lexes and parses text to produce a document.
    /// Choosing to parse with markdown may have a performance penalty
    pub fn new(text: &str, parser: Box<dyn Parser>) -> Self {
        let source: Vec<_> = text.chars().collect();

        Self::new_from_vec(Lrc::new(source), parser)
    }

    pub fn new_from_vec(source: Lrc<Vec<char>>, parser: Box<dyn Parser>) -> Self {
        let mut doc = Self {
            source,
            tokens: Vec::new(),
            parser
        };
        doc.parse();

        doc
    }

    pub fn new_plain_english(text: &str) -> Self {
        Self::new(text, Box::new(PlainEnglish))
    }

    pub fn new_markdown(text: &str) -> Self {
        Self::new(text, Box::new(Markdown))
    }

    /// Re-parse important language constructs.
    ///
    /// Should be run after every change to the underlying [`Self::source`].
    fn parse(&mut self) {
        self.tokens = self.parser.parse(&self.source);
        self.condense_spaces();
        self.condense_newlines();
        self.condense_contractions();
        self.condense_number_suffixes();
        self.match_quotes();
    }

    /// Given a list of indices, this function removes the subsequent
    /// `stretch_len - 1` elements after each index.
    ///
    /// Will extend token spans to include removed elements.
    /// Assumes condensed tokens are contiguous in source text.
    fn condense_indices(&mut self, indices: &[usize], stretch_len: usize) {
        // Update spans
        for idx in indices {
            let end_tok = self.tokens[idx + stretch_len - 1];
            let start_tok = &mut self.tokens[*idx];

            start_tok.span.end = end_tok.span.end;
        }

        // Trim
        let old = self.tokens.clone();
        self.tokens.clear();

        // Keep first chunk.
        self.tokens
            .extend_from_slice(&old[0..indices.first().copied().unwrap_or(indices.len())]);

        let mut iter = indices.iter().peekable();

        while let (Some(a_idx), b) = (iter.next(), iter.peek()) {
            self.tokens.push(old[*a_idx]);

            if let Some(b_idx) = b {
                self.tokens
                    .extend_from_slice(&old[a_idx + stretch_len..**b_idx]);
            }
        }

        // Keep last chunk.
        self.tokens.extend_from_slice(
            &old[indices
                .last()
                .map(|v| v + stretch_len)
                .unwrap_or(indices.len())..]
        );
    }

    pub fn get_token_at_char_index(&self, char_index: usize) -> Option<Token> {
        let index = self
            .tokens
            .binary_search_by(|t| {
                if t.span.overlaps_with(Span::new_with_len(char_index, 1)) {
                    Ordering::Equal
                } else {
                    t.span.start.cmp(&char_index)
                }
            })
            .ok()?;

        Some(self.tokens[index])
    }

    /// Defensively attempt to grab a specific token.
    pub fn get_token(&self, index: usize) -> Option<Token> {
        self.tokens.get(index).copied()
    }

    pub fn tokens(&self) -> impl Iterator<Item = Token> + '_ {
        self.tokens.iter().copied()
    }

    pub fn fat_tokens(&self) -> impl Iterator<Item = FatToken> + '_ {
        self.tokens().map(|token| token.to_fat(&self.source))
    }

    /// Iterate over the locations of punctuation the separates chunks.
    fn chunk_terminators(&self) -> impl Iterator<Item = usize> + '_ {
        self.tokens.iter().enumerate().filter_map(|(index, token)| {
            if is_chunk_terminator(&token.kind) {
                return Some(index);
            }
            None
        })
    }

    /// Get the index of the last chunk terminator.
    fn last_chunk_terminator(&self) -> Option<usize> {
        self.tokens
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, token)| {
                if is_chunk_terminator(&token.kind) {
                    return Some(index);
                }
                None
            })
    }

    /// Iterate over sentence chunks.
    pub fn chunks(&self) -> impl Iterator<Item = &'_ [Token]> + '_ {
        let first_sentence = self
            .chunk_terminators()
            .next()
            .map(|first_term| &self.tokens[0..=first_term]);

        let rest = self
            .chunk_terminators()
            .tuple_windows()
            .map(move |(a, b)| &self.tokens[a + 1..=b]);

        let last = if let Some(last_i) = self.last_chunk_terminator() {
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

    /// Iterate over the locations of the sentence terminators in the document.
    fn sentence_terminators(&self) -> impl Iterator<Item = usize> + '_ {
        self.tokens.iter().enumerate().filter_map(|(index, token)| {
            if is_sentence_terminator(&token.kind) {
                return Some(index);
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
                if is_sentence_terminator(&token.kind) {
                    return Some(index);
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

    pub fn get_span_content(&self, span: Span) -> &[char] {
        span.get_content(&self.source)
    }

    pub fn get_span_content_str(&self, span: Span) -> String {
        String::from_iter(self.get_span_content(span))
    }

    pub fn get_full_string(&self) -> String {
        self.get_span_content_str(Span {
            start: 0,
            end: self.source.len()
        })
    }

    pub fn get_full_content(&self) -> &[char] {
        &self.source
    }

    pub fn apply_suggestion(&mut self, suggestion: &Suggestion, span: Span) {
        let source = Lrc::make_mut(&mut self.source);

        match suggestion {
            Suggestion::ReplaceWith(chars) => {
                // Avoid allocation if possible
                if chars.len() == span.len() {
                    for (index, c) in chars.iter().enumerate() {
                        source[index + span.start] = *c
                    }
                } else {
                    let popped = source.split_off(span.start);

                    source.extend(chars);
                    source.extend(popped.into_iter().skip(span.len()));
                }
            }
            Suggestion::Remove => {
                for i in span.end..source.len() {
                    source[i - span.len()] = source[i];
                }

                source.truncate(source.len() - span.len());
            }
        }

        self.parse();
    }

    /// Searches for quotation marks and fills the
    /// [`Punctuation::Quote::twin_loc`] field. This is on a best effort
    /// basis.
    ///
    /// Current algorithm is very basic and could use some work.
    fn match_quotes(&mut self) {
        let quote_indices: Vec<usize> = self.tokens.iter_quote_indices().collect();

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

    /// Searches for number suffixes and condenses them down into single tokens
    fn condense_number_suffixes(&mut self) {
        if self.tokens.len() < 2 {
            return;
        }

        let mut replace_starts = Vec::new();

        for idx in 0..self.tokens.len() - 1 {
            let b = self.tokens[idx + 1];
            let a = self.tokens[idx];

            // TODO: Allow spaces between `a` and `b`

            if let (TokenKind::Number(..), TokenKind::Word) = (a.kind, b.kind) {
                if let Some(found_suffix) = NumberSuffix::from_chars(self.get_span_content(b.span))
                {
                    *self.tokens[idx].kind.as_mut_number().unwrap().1 = Some(found_suffix);
                    replace_starts.push(idx);
                }
            }
        }

        self.condense_indices(&replace_starts, 2);
    }

    /// Searches for multiple sequential newline tokens and condenses them down
    /// into one.
    fn condense_spaces(&mut self) {
        let mut cursor = 0;
        let copy = self.tokens.clone();

        let mut remove_these = VecDeque::new();

        while cursor < self.tokens.len() {
            // Locate a stretch of one or more newline tokens.
            let start_tok = &mut self.tokens[cursor];

            if let TokenKind::Space(start_count) = &mut start_tok.kind {
                loop {
                    cursor += 1;

                    if cursor >= copy.len() {
                        break;
                    }

                    let child_tok = &copy[cursor];
                    if let TokenKind::Space(n) = child_tok.kind {
                        *start_count += n;
                        start_tok.span.end = child_tok.span.end;
                        remove_these.push_back(cursor);
                        cursor += 1;
                    } else {
                        break;
                    };
                }
            }

            cursor += 1;
        }

        remove_indices(&mut self.tokens, remove_these);
    }

    /// Searches for multiple sequential newline tokens and condenses them down
    /// into one.
    fn condense_newlines(&mut self) {
        let mut cursor = 0;
        let copy = self.tokens.clone();

        let mut remove_these = VecDeque::new();

        while cursor < self.tokens.len() {
            // Locate a stretch of one or more newline tokens.
            let start_tok = &mut self.tokens[cursor];

            if let TokenKind::Newline(start_count) = &mut start_tok.kind {
                loop {
                    cursor += 1;

                    if cursor >= copy.len() {
                        break;
                    }

                    let child_tok = &copy[cursor];
                    if let TokenKind::Newline(n) = child_tok.kind {
                        *start_count += n;
                        start_tok.span.end = child_tok.span.end;
                        remove_these.push_back(cursor);
                        cursor += 1;
                    } else {
                        break;
                    };
                }
            }

            cursor += 1;
        }

        remove_indices(&mut self.tokens, remove_these);
    }

    /// Searches for contractions and condenses them down into single
    /// tokens.
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

        self.condense_indices(&replace_starts, 3);
    }
}

macro_rules! create_fns_on_doc {
    ($thing:ident) => {
        paste! {
            fn [< first_ $thing >](&self) -> Option<Token> {
                self.tokens.[< first_ $thing >]()
            }

            fn [<iter_ $thing _indices>](&self) -> impl Iterator<Item = usize> + '_ {
                self.tokens.[< iter_ $thing _indices >]()
            }

            fn [<iter_ $thing s>](&self) -> impl Iterator<Item = Token> + '_ {
                self.tokens.[< iter_ $thing s >]()
            }
        }
    };
}

impl TokenStringExt for Document {
    create_fns_on_doc!(word);
    create_fns_on_doc!(space);
    create_fns_on_doc!(apostrophe);
    create_fns_on_doc!(quote);
    create_fns_on_doc!(number);
    create_fns_on_doc!(at);

    fn first_sentence_word(&self) -> Option<Token> {
        self.tokens.first_sentence_word()
    }

    fn first_non_whitespace(&self) -> Option<Token> {
        self.tokens.first_non_whitespace()
    }

    fn span(&self) -> Option<Span> {
        self.tokens.span()
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

fn is_chunk_terminator(token: &TokenKind) -> bool {
    if is_sentence_terminator(token) {
        return true;
    }

    match token {
        TokenKind::Punctuation(punct) => [Punctuation::Comma].contains(punct),
        _ => false
    }
}

fn is_sentence_terminator(token: &TokenKind) -> bool {
    match token {
        TokenKind::Punctuation(punct) => [
            Punctuation::Period,
            Punctuation::Bang,
            Punctuation::Question
        ]
        .contains(punct),
        TokenKind::Newline(count) => *count >= 2,
        _ => false
    }
}

/// Removes a list of indices from a Vector.
/// Assumes that the provided indices are already in sorted order.
fn remove_indices<T>(vec: &mut Vec<T>, mut to_remove: VecDeque<usize>) {
    let mut i = 0;

    let mut next_remove = to_remove.pop_front();

    vec.retain(|_| {
        let keep = if let Some(next_remove) = next_remove {
            i != next_remove
        } else {
            true
        };

        if !keep {
            next_remove = to_remove.pop_front();
        }

        i += 1;
        keep
    });
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::Document;
    use crate::document::remove_indices;
    use crate::parsers::{Markdown, PlainEnglish};
    use crate::token::TokenStringExt;
    use crate::{Span, Token, TokenKind};

    #[test]
    fn parses_sentences_correctly() {
        let text = "There were three little pigs. They built three little homes.";
        let document = Document::new(text, Box::new(PlainEnglish));

        let mut sentence_strs = vec![];

        for sentence in document.sentences() {
            if let Some(span) = sentence.span() {
                sentence_strs.push(document.get_span_content_str(span));
            }
        }

        assert_eq!(
            sentence_strs,
            vec![
                "There were three little pigs.",
                " They built three little homes."
            ]
        )
    }

    fn assert_condensed_contractions(text: &str, final_tok_count: usize) {
        let document = Document::new(text, Box::new(PlainEnglish));

        assert_eq!(document.tokens.len(), final_tok_count);

        let markdown_parser = Markdown;
        let document = Document::new(text, Box::new(markdown_parser));

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
    fn selects_token_at_char_index() {
        let text = "There were three little pigs. They built three little homes.";
        let document = Document::new(text, Box::new(PlainEnglish));

        assert_eq!(
            document.get_token_at_char_index(19),
            Some(Token {
                kind: TokenKind::Word,
                span: Span::new(17, 23)
            })
        )
    }

    #[test]
    fn condenses_number_suffixes() {
        fn assert_token_count(source: &str, count: usize) {
            let document = Document::new_plain_english(source);
            assert_eq!(document.tokens.len(), count);
        }

        assert_token_count("1st", 1);
        assert_token_count("This is the 2nd test", 9);
        assert_token_count("This is the 3rd test", 9);
        assert_token_count(
            "It works even with weird capitalization like this: 600nD",
            18
        );
    }

    #[test]
    fn removes_requested_indices() {
        let mut data: Vec<i32> = (0..10).collect();
        let remove: VecDeque<usize> = vec![1, 4, 6].into_iter().collect();

        remove_indices(&mut data, remove);

        assert_eq!(data, vec![0, 2, 3, 5, 7, 8, 9])
    }
}

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::Display;

use paste::paste;

use crate::parsers::{Markdown, Parser, PlainEnglish};
use crate::patterns::{PatternExt, RepeatingPattern, SequencePattern};
use crate::punctuation::Punctuation;
use crate::span::Span;
use crate::token::NumberSuffix;
use crate::vec_ext::VecExt;
use crate::{Dictionary, FatToken, FullDictionary, Lrc, Token, TokenKind, TokenStringExt};

/// A document containing some amount of lexed and parsed English text.
#[derive(Debug, Clone)]
pub struct Document {
    source: Lrc<Vec<char>>,
    tokens: Vec<Token>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new("", &mut PlainEnglish, &FullDictionary::curated())
    }
}

impl Document {
    /// Lexes and parses text to produce a document using a provided language
    /// parser and dictionary.
    pub fn new(text: &str, parser: &mut impl Parser, dictionary: &impl Dictionary) -> Self {
        let source: Vec<_> = text.chars().collect();

        Self::new_from_vec(Lrc::new(source), parser, dictionary)
    }

    /// Lexes and parses text to produce a document using a provided language
    /// parser and the included curated dictionary.
    pub fn new_curated(text: &str, parser: &mut impl Parser) -> Self {
        let source: Vec<_> = text.chars().collect();

        Self::new_from_vec(Lrc::new(source), parser, &FullDictionary::curated())
    }

    /// Lexes and parses text to produce a document using a provided language
    /// parser and dictionary.
    pub fn new_from_vec(
        source: Lrc<Vec<char>>,
        parser: &mut impl Parser,
        dictionary: &impl Dictionary,
    ) -> Self {
        let tokens = parser.parse(&source);

        let mut document = Self { source, tokens };
        document.parse(dictionary);

        document
    }

    /// Parse text to produce a document using the built-in [`PlainEnglish`]
    /// parser and curated dictionary.
    pub fn new_plain_english_curated(text: &str) -> Self {
        Self::new(text, &mut PlainEnglish, &FullDictionary::curated())
    }

    /// Parse text to produce a document using the built-in [`PlainEnglish`]
    /// parser and a provided dictionary.
    pub fn new_plain_english(text: &str, dictionary: &impl Dictionary) -> Self {
        Self::new(text, &mut PlainEnglish, dictionary)
    }

    /// Parse text to produce a document using the built-in [`Markdown`] parser
    /// and curated dictionary.
    pub fn new_markdown_curated(text: &str) -> Self {
        Self::new(text, &mut Markdown, &FullDictionary::curated())
    }

    /// Parse text to produce a document using the built-in [`PlainEnglish`]
    /// parser and the curated dictionary.
    pub fn new_markdown(text: &str, dictionary: &impl Dictionary) -> Self {
        Self::new(text, &mut Markdown, dictionary)
    }

    /// Re-parse important language constructs.
    ///
    /// Should be run after every change to the underlying [`Self::source`].
    fn parse(&mut self, dictionary: &impl Dictionary) {
        self.condense_spaces();
        self.condense_ellipsis();
        self.condense_newlines();
        self.newlines_to_breaks();
        self.condense_contractions();
        self.condense_dotted_initialisms();
        self.condense_number_suffixes();
        self.match_quotes();

        for token in self.tokens.iter_mut() {
            if let TokenKind::Word(meta) = &mut token.kind {
                let word_source = token.span.get_content(&self.source);
                let found_meta = dictionary.get_word_metadata(word_source);
                *meta = meta.or(&found_meta);
            }
        }
    }

    /// Convert all sets of newlines greater than 2 to paragraph breaks.
    fn newlines_to_breaks(&mut self) {
        for token in &mut self.tokens {
            if let TokenKind::Newline(n) = token.kind {
                if n >= 2 {
                    token.kind = TokenKind::ParagraphBreak;
                }
            }
        }
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
                .unwrap_or(indices.len())..],
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

    /// Get an iterator over all the tokens contained in the document.
    pub fn tokens(&self) -> impl Iterator<Item = Token> + '_ {
        self.tokens.iter().copied()
    }

    /// Get an iterator over all the tokens contained in the document.
    pub fn fat_tokens(&self) -> impl Iterator<Item = FatToken> + '_ {
        self.tokens().map(|token| token.to_fat(&self.source))
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

    pub fn get_full_content(&self) -> &[char] {
        &self.source
    }

    pub fn get_source(&self) -> &[char] {
        &self.source
    }

    pub fn get_tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// Searches for quotation marks and fills the
    /// [`Punctuation::Quote::twin_loc`] field. This is on a best effort
    /// basis.
    ///
    /// Current algorithm is basic and could use some work.
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

            if let (TokenKind::Number(..), TokenKind::Word(..)) = (a.kind, b.kind) {
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

        self.tokens.remove_indices(remove_these);
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

        self.tokens.remove_indices(remove_these);
    }

    /// Condenses words like "i.e.", "e.g." and "N.S.A." down to single words
    /// using a state machine.
    fn condense_dotted_initialisms(&mut self) {
        if self.tokens.len() < 2 {
            return;
        }

        let mut to_remove = VecDeque::new();

        let mut cursor = 1;

        let mut initialism_start = None;

        loop {
            let a = self.tokens[cursor - 1];
            let b = self.tokens[cursor];

            let is_initialism_chunk = a.kind.is_word() && a.span.len() == 1 && b.kind.is_period();

            if is_initialism_chunk {
                if initialism_start.is_none() {
                    initialism_start = Some(cursor - 1);
                } else {
                    to_remove.push_back(cursor - 1);
                }

                to_remove.push_back(cursor);
                cursor += 1;
            } else {
                if let Some(start) = initialism_start {
                    let end = self.tokens[cursor - 2].span.end;
                    let start_tok: &mut Token = &mut self.tokens[start];
                    start_tok.span.end = end;
                }

                initialism_start = None;
            }

            cursor += 1;

            if cursor >= self.tokens.len() - 1 {
                break;
            }
        }

        self.tokens.remove_indices(to_remove);
    }

    fn uncached_ellipsis_pattern() -> Lrc<RepeatingPattern> {
        let period = SequencePattern::default().then_period();
        Lrc::new(RepeatingPattern::new(Box::new(period)))
    }

    thread_local! {
        static ELLIPSIS_PATTERN: Lrc<RepeatingPattern> = Document::uncached_ellipsis_pattern();
    }

    fn condense_ellipsis(&mut self) {
        let found = Self::ELLIPSIS_PATTERN
            .with(|v| v.clone())
            .find_all_matches(&self.tokens, &self.source);
        let mut to_remove = VecDeque::new();

        for found_slice in found {
            if found_slice.len() <= 1 {
                continue;
            }

            let found_toks = &mut self.tokens[found_slice.start..found_slice.end];

            let end_char = found_toks.last().unwrap().span.end;
            let first = found_toks.first_mut().unwrap();
            first.kind = TokenKind::Punctuation(Punctuation::Ellipsis);
            first.span.end = end_char;
            for i in found_slice.start + 1..found_slice.end {
                to_remove.push_back(i)
            }
        }

        self.tokens.remove_indices(to_remove);
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
                    TokenKind::Word(..),
                    TokenKind::Punctuation(Punctuation::Apostrophe),
                    TokenKind::Word(..)
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

/// Creates functions necessary to implement [`TokenStringExt]` on a document.
macro_rules! create_fns_on_doc {
    ($thing:ident) => {
        paste! {
            fn [< first_ $thing >](&self) -> Option<Token> {
                self.tokens.[< first_ $thing >]()
            }

            fn [< last_ $thing >](&self) -> Option<Token> {
                self.tokens.[< last_ $thing >]()
            }

            fn [< last_ $thing _index>](&self) -> Option<usize> {
                self.tokens.[< last_ $thing _index >]()
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
    create_fns_on_doc!(pipe);
    create_fns_on_doc!(quote);
    create_fns_on_doc!(number);
    create_fns_on_doc!(at);
    create_fns_on_doc!(ellipsis);
    create_fns_on_doc!(unlintable);
    create_fns_on_doc!(sentence_terminator);
    create_fns_on_doc!(chunk_terminator);
    create_fns_on_doc!(punctuation);

    fn first_sentence_word(&self) -> Option<Token> {
        self.tokens.first_sentence_word()
    }

    fn first_non_whitespace(&self) -> Option<Token> {
        self.tokens.first_non_whitespace()
    }

    fn span(&self) -> Option<Span> {
        self.tokens.span()
    }

    fn iter_linking_verb_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.tokens.iter_linking_verb_indices()
    }

    fn iter_linking_verbs(&self) -> impl Iterator<Item = Token> + '_ {
        self.tokens.iter_linking_verbs()
    }

    fn iter_chunks(&self) -> impl Iterator<Item = &'_ [Token]> + '_ {
        self.tokens.iter_chunks()
    }

    fn iter_sentences(&self) -> impl Iterator<Item = &'_ [Token]> + '_ {
        self.tokens.iter_sentences()
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

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::Document;
    use crate::Span;

    fn assert_condensed_contractions(text: &str, final_tok_count: usize) {
        let document = Document::new_plain_english_curated(text);

        assert_eq!(document.tokens.len(), final_tok_count);

        let document = Document::new_markdown_curated(text);

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
        let document = Document::new_plain_english_curated(text);

        let got = document.get_token_at_char_index(19).unwrap();

        assert!(got.kind.is_word());
        assert_eq!(got.span, Span::new(17, 23));
    }

    fn assert_token_count(source: &str, count: usize) {
        let document = Document::new_plain_english_curated(source);

        dbg!(document.tokens().map(|t| t.kind).collect_vec());
        assert_eq!(document.tokens.len(), count);
    }

    #[test]
    fn condenses_number_suffixes() {
        assert_token_count("1st", 1);
        assert_token_count("This is the 2nd test", 9);
        assert_token_count("This is the 3rd test", 9);
        assert_token_count(
            "It works even with weird capitalization like this: 600nD",
            18,
        );
    }

    #[test]
    fn condenses_ie() {
        assert_token_count("There is a thing (i.e. that one)", 15);
        assert_token_count("We are trying to condense \"i.e.\"", 13);
        assert_token_count(r#"Condenses words like "i.e.", "e.g." and "N.S.A.""#, 20);
    }

    #[test]
    fn condenses_eg() {
        assert_token_count("We are trying to condense \"e.g.\"", 13);
        assert_token_count(r#"Condenses words like "i.e.", "e.g." and "N.S.A.""#, 20);
    }

    #[test]
    fn condenses_nsa() {
        assert_token_count(r#"Condenses words like "i.e.", "e.g." and "N.S.A.""#, 20);
    }

    #[test]
    fn parses_ellipsis() {
        assert_token_count("...", 1);
    }

    #[test]
    fn parses_long_ellipsis() {
        assert_token_count(".....", 1);
    }

    #[test]
    fn parses_short_ellipsis() {
        assert_token_count("..", 1);
    }
}

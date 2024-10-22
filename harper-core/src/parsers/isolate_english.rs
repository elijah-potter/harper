use crate::{language_detection::is_likely_english, Dictionary};

use super::{Parser, Token, TokenStringExt};

/// A parser that wraps another, using heuristics to quickly redact paragraphs of a document that aren't
/// intended to be English text.
pub struct IsolateEnglish<D: Dictionary> {
    inner: Box<dyn Parser>,
    dict: D,
}

impl<D: Dictionary> IsolateEnglish<D> {
    pub fn new(inner: Box<dyn Parser>, dictionary: D) -> Self {
        Self {
            inner,
            dict: dictionary,
        }
    }
}

impl<D: Dictionary> Parser for IsolateEnglish<D> {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let tokens = self.inner.parse(source);

        let mut english_tokens: Vec<Token> = Vec::with_capacity(tokens.len());

        for sentence in tokens.iter_sentences() {
            if sentence.len() > 5 && is_likely_english(sentence, source, &self.dict) {
                english_tokens.extend(sentence);
            }
        }

        english_tokens
    }
}

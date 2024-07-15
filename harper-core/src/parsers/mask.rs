use super::Parser;
use crate::mask::Masker;
use crate::{Token, TokenKind};

/// Composes a Masker and a Parser to parse only masked chunks of text.
pub struct Mask<M, P>
where
    M: Masker,
    P: Parser
{
    pub masker: M,
    pub parser: P
}

impl<M, P> Mask<M, P>
where
    M: Masker,
    P: Parser
{
    pub fn new(masker: M, parser: P) -> Self {
        Self { masker, parser }
    }
}

impl<M, P> Parser for Mask<M, P>
where
    M: Masker,
    P: Parser
{
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mask = self.masker.create_mask(source);

        let mut tokens = Vec::new();

        for (span, content) in mask.iter_allowed(source) {
            let new_tokens = &mut self.parser.parse(content);

            if let Some(last) = new_tokens.last_mut() {
                if let TokenKind::Newline(n) = &mut last.kind {
                    if *n == 1 {
                        *n = 2;
                    }
                }
            }

            for token in new_tokens.iter_mut() {
                token.span.push_by(span.start);
            }

            tokens.append(new_tokens);
        }

        tokens
    }
}

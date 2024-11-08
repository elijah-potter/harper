use harper_data::{Span, Token, TokenKind};

use super::Parser;
use crate::Masker;

/// Composes a Masker and a Parser to parse only masked chunks of text.
pub struct MaskParser<M, P>
where
    M: Masker,
    P: Parser,
{
    pub masker: M,
    pub parser: P,
}

impl<M, P> MaskParser<M, P>
where
    M: Masker,
    P: Parser,
{
    pub fn new(masker: M, parser: P) -> Self {
        Self { masker, parser }
    }
}

impl<M, P> Parser for MaskParser<M, P>
where
    M: Masker,
    P: Parser,
{
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mask = self.masker.create_mask(source);

        let mut tokens: Vec<Token> = Vec::new();

        let mut last_allowed: Option<Span> = None;

        for (span, content) in mask.iter_allowed(source) {
            // Check if there was a line break between the last chunk.
            if let Some(last_allowed) = last_allowed {
                let intervening = Span::new(last_allowed.end, span.start);

                if intervening.get_content(source).contains(&'\n') {
                    tokens.push(Token::new(intervening, TokenKind::ParagraphBreak))
                }
            }

            let new_tokens = &mut self.parser.parse(content);

            for token in new_tokens.iter_mut() {
                token.span.push_by(span.start);
            }

            tokens.append(new_tokens);
            last_allowed = Some(span);
        }

        tokens
    }
}

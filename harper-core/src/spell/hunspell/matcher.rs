use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// A simplified, Regex-like matcher.
///
/// See Hunspell documentation on affixes for more information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Matcher {
    /// Position-based operators.
    operators: Vec<Operator>
}

impl Matcher {
    pub fn parse(source: &str) -> Result<Self, Error> {
        let mut operators = Vec::new();

        let char_indices: Vec<_> = source.char_indices().collect();
        let mut char_idx = 0;

        while char_idx < char_indices.len() {
            let (idx, c) = char_indices[char_idx];

            match c {
                '[' => {
                    let close_idx = source[idx..]
                        .find(']')
                        .ok_or(Error::UnmatchedBracket { index: idx })?;

                    let bracket_contents = &source[idx + 1..close_idx];

                    let invert = matches!(bracket_contents.chars().next(), Some('^'));

                    if invert {
                        let chars: Vec<char> = bracket_contents.chars().skip(1).collect();
                        char_idx += chars.len() + 2;
                        operators.push(Operator::MatchNone(chars));
                    } else {
                        let chars: Vec<char> = bracket_contents.chars().collect();
                        char_idx += chars.len() + 1;
                        operators.push(Operator::MatchOne(chars));
                    }
                }
                '.' => operators.push(Operator::Any),
                _ => operators.push(Operator::Literal(c))
            }

            char_idx += 1;
        }

        Ok(Self { operators })
    }

    pub fn len(&self) -> usize {
        self.operators.len()
    }

    pub fn matches(&self, chars: &[char]) -> bool {
        if chars.len() != self.len() {
            return false;
        }

        for (c, op) in chars.iter().zip(self.operators.iter()) {
            if !op.matches(*c) {
                return false;
            }
        }

        true
    }
}

impl Display for Matcher {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for op in &self.operators {
            match op {
                Operator::Literal(c) => write!(f, "{}", c)?,
                Operator::MatchOne(cs) => {
                    write!(f, "[")?;

                    for c in cs {
                        write!(f, "{}", c)?;
                    }

                    write!(f, "]")?;
                }
                Operator::MatchNone(cs) => {
                    write!(f, "[^")?;

                    for c in cs {
                        write!(f, "{}", c)?;
                    }

                    write!(f, "]")?;
                }
                Operator::Any => write!(f, ".")?
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum Operator {
    Literal(char),
    MatchOne(Vec<char>),
    MatchNone(Vec<char>),
    Any
}

impl Operator {
    fn matches(&self, a: char) -> bool {
        match self {
            Operator::Literal(b) => a == *b,
            Operator::MatchOne(b) => b.contains(&a),
            Operator::MatchNone(b) => !b.contains(&a),
            Operator::Any => true
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error {
    #[error("Unmatched bracket at index: {index}")]
    UnmatchedBracket { index: usize }
}

#[cfg(test)]
mod tests {
    use super::Matcher;
    use crate::spell::hunspell::matcher::Operator;

    #[test]
    fn parses_simple() {
        let matcher = Matcher::parse("[^aeiou]a.s").unwrap();
        assert_eq!(
            matcher.operators,
            vec![
                Operator::MatchNone(vec!['a', 'e', 'i', 'o', 'u']),
                Operator::Literal('a'),
                Operator::Any,
                Operator::Literal('s')
            ]
        )
    }

    #[test]
    fn matches_vowels() {
        let matcher = Matcher::parse("[aeiou]").unwrap();

        assert!(matcher.matches(&['a']));
        assert!(matcher.matches(&['e']));
        assert!(matcher.matches(&['i']));
        assert!(matcher.matches(&['o']));
        assert!(matcher.matches(&['u']));
    }

    #[test]
    fn round_trip() {
        let source = "[^aeiou]a.s";
        let matcher = Matcher::parse(source).unwrap();

        assert_eq!(matcher.to_string(), source);
    }
}

use itertools::Itertools;
use smallvec::ToSmallVec;
use std::usize;

use hashbrown::HashMap;

use crate::{spell::DictWord, Span};

use super::{matcher::Matcher, word_list::MarkedWord, Error};

#[derive(Debug, Clone)]
struct AffixReplacement {
    pub remove: Vec<char>,
    pub add: Vec<char>,
    pub condition: Matcher,
}

#[derive(Debug, Clone)]
struct Expansion {
    // If not true, its a prefix
    pub suffix: bool,
    pub cross_product: bool,
    pub replacements: Vec<AffixReplacement>,
}

#[derive(Debug)]
pub struct AttributeList {
    /// Key = Affix Flag
    affixes: HashMap<char, Expansion>,
}

impl AttributeList {
    pub fn parse(file: &str) -> Result<Self, Error> {
        let mut output = Self {
            affixes: HashMap::default(),
        };

        for line in file.lines() {
            if line.chars().filter(|c| !c.is_whitespace()).count() == 0 {
                continue;
            }

            output.parse_line(line)?;
        }

        Ok(output)
    }

    fn parse_line(&mut self, line: &str) -> Result<(), Error> {
        if line.len() < 4 {
            return Ok(());
        }

        let mut parser = AttributeArgParser::new(line);

        let suffix = match parser.parse_arg()? {
            "PFX" => false,
            "SFX" => true,
            _ => return Ok(()),
        };

        let flag = {
            let flag_arg = parser.parse_arg()?;
            if flag_arg.len() != 1 {
                return Err(Error::MultiCharacterFlag);
            };

            flag_arg.chars().next().unwrap()
        };

        if let Some(expansion) = self.affixes.get_mut(&flag) {
            let remove_arg = parser.parse_arg()?;

            let remove: Vec<_> = remove_arg.chars().collect();
            let remove = if remove.len() == 1 && remove[0] == '0' {
                vec![]
            } else {
                remove
            };

            let add = parser.parse_arg()?.chars().collect();
            let condition = Matcher::parse(parser.parse_arg()?)?;

            let replacement = AffixReplacement {
                remove,
                add,
                condition,
            };

            expansion.replacements.push(replacement)
        } else {
            let cross_product = parser.parse_bool_arg()?;
            let count = parser.parse_usize_arg()?;

            self.affixes.insert(
                flag,
                Expansion {
                    suffix,
                    cross_product,
                    replacements: Vec::with_capacity(count),
                },
            );
        }

        Ok(())
    }

    /// Expand [`MarkedWord`] into a list of full words, including itself.
    ///
    /// In the future, I want to make this function cleaner and faster.
    pub fn expand_marked_word(&self, word: MarkedWord) -> Result<Vec<DictWord>, Error> {
        let mut words = Vec::with_capacity(word.attributes.len() + 1);

        for attr in &word.attributes {
            let Some(expansion) = self.affixes.get(attr) else {
                continue;
            };

            let mut new_words = Vec::new();

            for replacement in &expansion.replacements {
                new_words.extend(Self::apply_replacement(
                    replacement,
                    &word.letters,
                    expansion.suffix,
                ))
            }

            if expansion.cross_product {
                let mut opp_attr = Vec::new();

                for attr in &word.attributes {
                    let Some(attr_def) = self.affixes.get(attr) else {
                        continue;
                    };
                    if attr_def.suffix != expansion.suffix {
                        opp_attr.push(*attr);
                    }
                }

                let mut cross_product_words = Vec::new();

                for new_word in new_words {
                    cross_product_words.extend(self.expand_marked_word(MarkedWord {
                        letters: new_word,
                        attributes: opp_attr.clone(),
                    })?)
                }

                words.extend_from_slice(&cross_product_words);
            } else {
                words.extend_from_slice(&new_words);
            }
        }

        words.push(word.letters);

        Ok(words)
    }

    pub fn expand_marked_words(
        &self,
        words: impl IntoIterator<Item = MarkedWord>,
    ) -> Result<Vec<DictWord>, Error> {
        let mut output = Vec::new();

        for word in words {
            output.extend(self.expand_marked_word(word)?.into_iter().unique());
        }

        Ok(output)
    }

    fn apply_replacement(
        replacement: &AffixReplacement,
        letters: &[char],
        suffix: bool,
    ) -> Option<DictWord> {
        if replacement.condition.len() > letters.len() {
            return None;
        }

        let target_span = if suffix {
            Span::new(letters.len() - replacement.condition.len(), letters.len())
        } else {
            Span::new(0, replacement.condition.len())
        };

        let target_segment = target_span.get_content(letters);

        if replacement.condition.matches(target_segment) {
            let mut replaced_segment = letters.to_smallvec();
            let mut remove: DictWord = replacement.remove.to_smallvec();

            if !suffix {
                replaced_segment.reverse();
            } else {
                remove.reverse();
            }

            for c in &remove {
                let Some(last) = replaced_segment.last() else {
                    return None;
                };

                if last == c {
                    replaced_segment.pop();
                } else {
                    return None;
                }
            }

            let mut to_add = replacement.add.to_vec();

            if !suffix {
                to_add.reverse()
            }

            replaced_segment.extend(to_add);

            if !suffix {
                replaced_segment.reverse();
            }

            return Some(replaced_segment);
        }

        None
    }
}

struct AttributeArgParser<'a> {
    line: &'a str,
    cursor: usize,
}

impl<'a> AttributeArgParser<'a> {
    pub fn new(line: &'a str) -> Self {
        Self { line, cursor: 0 }
    }

    // Grap next affix argument, returning an error if it doesn't exist.
    fn parse_arg(&mut self) -> Result<&'a str, Error> {
        let Some((next_word_start, _)) = self.line[self.cursor..]
            .char_indices()
            .find(|(_i, c)| !c.is_whitespace())
        else {
            dbg!(self.cursor);
            return Err(Error::UnexpectedEndOfLine);
        };

        let next_word_end = self.line[self.cursor + next_word_start..]
            .char_indices()
            .find(|(_i, c)| c.is_whitespace())
            .map(|(end, _)| end)
            .unwrap_or(self.line.len() - self.cursor - next_word_start);

        let abs_start = next_word_start + self.cursor;
        let abs_end = next_word_start + self.cursor + next_word_end;

        self.cursor = abs_end;

        Ok(&self.line[abs_start..abs_end])
    }

    // Grab next affix argument, returning an error if it isn't parsable as a number.
    fn parse_usize_arg(&mut self) -> Result<usize, Error> {
        self.parse_arg()?
            .parse()
            .map_err(|_| Error::ExpectedUnsignedInteger)
    }

    // Grab next affix argument, returning an error if it isn't Y or N.
    fn parse_bool_arg(&mut self) -> Result<bool, Error> {
        match self.parse_arg()? {
            "Y" => Ok(true),
            "N" => Ok(false),
            _ => Err(Error::ExpectedBoolean),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::ATTR_LIST;
    use super::AttributeList;

    #[test]
    fn can_parse_test_file() {
        AttributeList::parse(ATTR_LIST).unwrap();
    }
}

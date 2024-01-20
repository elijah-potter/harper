use super::Error;

pub struct MarkedWord {
    pub letters: Vec<char>,
    pub attributes: Vec<char>,
}

pub struct Dictionary {
    pub(self) words: Vec<MarkedWord>,
}

/// Parse a hunspell word list
///
/// Returns [`None`] if the given string is invalid.
pub fn parse_word_list(source: &str) -> Result<Vec<MarkedWord>, Error> {
    let mut lines = source.lines();

    let approx_item_count = lines
        .next()
        .ok_or(Error::MalformedItemCount)?
        .parse()
        .map_err(|_| Error::MalformedItemCount)?;

    let mut words = Vec::with_capacity(approx_item_count);

    for line in lines {
        if let Some((word, attributes)) = line.split_once('/') {
            words.push(MarkedWord {
                letters: word.chars().collect(),
                attributes: attributes.chars().collect(),
            })
        } else {
            words.push(MarkedWord {
                letters: line.chars().collect(),
                attributes: Vec::new(),
            })
        }
    }

    Ok(words)
}

#[cfg(test)]
mod tests {
    use super::super::tests::TEST_WORD_LIST;
    use super::parse_word_list;

    #[test]
    fn can_parse_test_file() {
        let list = parse_word_list(TEST_WORD_LIST).unwrap();

        assert_eq!(list.last().unwrap().attributes.len(), 2);
        assert_eq!(list.len(), 3);
    }
}

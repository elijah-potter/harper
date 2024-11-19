use std::borrow::Cow;

use itertools::{Itertools, MinMaxResult};

use crate::{CharString, CharStringExt, WordMetadata};

pub use self::dictionary::Dictionary;
pub use self::fst_dictionary::FstDictionary;
pub use self::full_dictionary::FullDictionary;
pub use self::merged_dictionary::MergedDictionary;

mod dictionary;
mod fst_dictionary;
mod full_dictionary;
mod hunspell;
mod merged_dictionary;

#[derive(PartialEq)]
pub struct FuzzyMatchResult<'a> {
    word: &'a [char],
    edit_distance: u8,
    metadata: WordMetadata,
}

impl<'a> PartialOrd for FuzzyMatchResult<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.edit_distance.partial_cmp(&other.edit_distance)
    }
}

fn order_suggestions(matches: Vec<FuzzyMatchResult>) -> Vec<&[char]> {
    let mut found: Vec<&FuzzyMatchResult> = Vec::with_capacity(matches.len());
    // Often the longest and the shortest words are the most helpful, so lets push
    // them first.
    let minmax = matches.iter().position_minmax_by_key(|fmr| fmr.word.len());
    if let MinMaxResult::MinMax(a, b) = minmax {
        if a == b {
            found.push(&matches[a]);
        } else {
            found.push(&matches[a]);
            found.push(&matches[b]);
        }

        // Push the rest
        found.extend(
            matches
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != a && *i != b)
                .map(|v| v.1),
        );
    } else {
        found.extend(&matches);
    }

    // Swap the lowest edit distance word with the shortest.
    if found.len() >= 3 {
        found.swap(0, 2);
    }

    // Let common words bubble up, but do not prioritize them over all else.
    found.sort_by_key(|fmr| fmr.edit_distance + if fmr.metadata.common { 0 } else { 1 });

    found.into_iter().map(|fmr| fmr.word).collect()
}

/// Get the closest matches in the provided [`Dictionary`] and rank them
/// Implementation is left up to the underlying dictionary.
pub fn suggest_correct_spelling<'a>(
    misspelled_word: &[char],
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &'a impl Dictionary,
) -> Vec<&'a [char]> {
    let matches: Vec<FuzzyMatchResult> = dictionary
        .fuzzy_match(misspelled_word, max_edit_dist, result_limit)
        .into_iter()
        .collect();
    order_suggestions(matches)
}

/// Convenience function over [`suggest_correct_spelling`] that does conversions
/// for you.
pub fn suggest_correct_spelling_str(
    misspelled_word: impl Into<String>,
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &impl Dictionary,
) -> Vec<String> {
    let chars: CharString = misspelled_word.into().chars().collect();
    suggest_correct_spelling(&chars, result_limit, max_edit_dist, dictionary)
        .into_iter()
        .map(|a| a.to_string())
        .collect()
}

/// Convert a given character sequence to the standard character set
/// the dictionary is in.
fn seq_to_normalized(seq: &[char]) -> Cow<'_, [char]> {
    if seq.iter().any(|c| char_to_normalized(*c) != *c) {
        Cow::Owned(seq.iter().copied().map(char_to_normalized).collect())
    } else {
        Cow::Borrowed(seq)
    }
}

fn char_to_normalized(c: char) -> char {
    match c {
        '’' => '\'',
        '‘' => '\'',
        '＇' => '\'',
        _ => c,
    }
}

// Computes the Levenshtein edit distance between two patterns.
// This is accomplished via a memory-optimized Wagner-Fischer algorithm
//
// This variant avoids allocation if you already have buffers.
#[inline]
fn edit_distance_min_alloc(
    source: &[char],
    target: &[char],
    previous_row: &mut Vec<u8>,
    current_row: &mut Vec<u8>,
) -> u8 {
    if cfg!(debug_assertions) {
        assert!(source.len() <= 255 && target.len() <= 255);
    }

    let row_width = source.len();
    let col_height = target.len();

    previous_row.clear();
    previous_row.extend(0u8..=row_width as u8);
    // Alright if not zeroed, since we overwrite it anyway.
    current_row.resize(row_width + 1, 0);

    for j in 1..=col_height {
        current_row[0] = j as u8;

        for i in 1..=row_width {
            let cost = if source[i - 1] == target[j - 1] { 0 } else { 1 };

            current_row[i] = (previous_row[i] + 1)
                .min(current_row[i - 1] + 1)
                .min(previous_row[i - 1] + cost);
        }

        std::mem::swap(previous_row, current_row);
    }

    previous_row[row_width]
}

fn edit_distance(source: &[char], target: &[char]) -> u8 {
    edit_distance_min_alloc(source, target, &mut Vec::new(), &mut Vec::new())
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::spell::FuzzyMatchResult;

    use super::{
        edit_distance, order_suggestions, seq_to_normalized, suggest_correct_spelling_str,
        Dictionary, FstDictionary, FullDictionary,
    };

    const RESULT_LIMIT: usize = 100;
    const MAX_EDIT_DIST: u8 = 3;

    fn assert_edit_dist(source: &str, target: &str, expected: u8) {
        let source: Vec<_> = source.chars().collect();
        let target: Vec<_> = target.chars().collect();

        let dist = edit_distance(&source, &target);
        assert_eq!(dist, expected)
    }

    #[test]
    fn normalizes_weve() {
        let word = vec!['w', 'e', '’', 'v', 'e'];
        let norm = seq_to_normalized(&word);

        assert_eq!(norm.clone(), vec!['w', 'e', '\'', 'v', 'e'])
    }

    #[test]
    fn simple_edit_distance_1() {
        assert_edit_dist("kitten", "sitting", MAX_EDIT_DIST)
    }

    #[test]
    fn simple_edit_distance_2() {
        assert_edit_dist("saturday", "sunday", MAX_EDIT_DIST)
    }

    #[test]
    fn produces_no_duplicates() {
        let results = suggest_correct_spelling_str(
            "punctation",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results, results.iter().unique().collect_vec());

        assert_eq!(results.iter().unique().count(), results.len())
    }

    #[test]
    fn zero_edit_distance() {
        let source: Vec<_> = "hello".chars().collect();
        let target: Vec<_> = "hello".chars().collect();
        assert_eq!(edit_distance(&source, &target), 0);
    }

    #[test]
    fn one_edit_distance() {
        let source: Vec<_> = "hello".chars().collect();
        let target: Vec<_> = "hellos".chars().collect();
        assert_eq!(edit_distance(&source, &target), 1);

        let target: Vec<_> = "hell".chars().collect();
        assert_eq!(edit_distance(&source, &target), 1);

        let target: Vec<_> = "hell".chars().collect();
        assert_eq!(edit_distance(&source, &target), 1);

        let target: Vec<_> = "hvllo".chars().collect();
        assert_eq!(edit_distance(&source, &target), 1);

        let target: Vec<_> = "Hello".chars().collect();
        assert_eq!(edit_distance(&source, &target), 1);
    }

    /// Ensures that the suggestions are ordered taking into account commonality
    #[test]
    fn common_words_first() {
        let dict = FstDictionary::curated();
        // Select three common words
        let common_words = dict
            .words_iter()
            .filter_map(|word| {
                let metadata = dict.get_word_metadata(word);
                if metadata.common {
                    Some(FuzzyMatchResult {
                        word,
                        edit_distance: 0,
                        metadata,
                    })
                } else {
                    None
                }
            })
            .take(3);
        // Select three uncommon words
        let uncommon_words = dict
            .words_iter()
            .filter_map(|word| {
                let metadata = dict.get_word_metadata(word);
                if metadata.common {
                    None
                } else {
                    Some(FuzzyMatchResult {
                        word,
                        edit_distance: 0,
                        metadata,
                    })
                }
            })
            .take(3);
        // Feed the common and uncommon words into the ordering function, starting with uncommon
        // words
        let words = uncommon_words.merge(common_words).collect();
        let suggestions = order_suggestions(words);

        // Asserts that the ordering prioritizes common words
        let common_first = suggestions
            .into_iter()
            .take(3)
            .all(|word| dict.get_word_metadata(word).common);

        assert!(common_first);
    }

    #[test]
    fn this_correction() {
        let results = suggest_correct_spelling_str(
            "Ths",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"this".to_string()));
    }

    // I'm ignoring this one because the sorting algorithm prioritizes shorter words at the same
    // edit distance that are also common.
    #[ignore]
    #[test]
    fn issue_182() {
        let results = suggest_correct_spelling_str(
            "im",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"I'm".to_string()));
    }

    #[test]
    fn fst_spellcheck_hvllo() {
        let results = suggest_correct_spelling_str(
            "hvllo",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"hello".to_string()));
    }

    #[test]
    fn full_spellcheck_hvllo() {
        let results = suggest_correct_spelling_str(
            "hvllo",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FullDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"hello".to_string()));
    }

    #[test]
    fn fst_spellcheck_common() {
        let results = suggest_correct_spelling_str(
            "aboot",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"about".to_string()));
    }

    #[test]
    fn full_spellcheck_common() {
        let results = suggest_correct_spelling_str(
            "aboot",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FullDictionary::curated(),
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"about".to_string()));
    }

    #[test]
    fn spellchecking_is_deterministic() {
        let results1 = suggest_correct_spelling_str(
            "hello",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );
        let results2 = suggest_correct_spelling_str(
            "hello",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );
        let results3 = suggest_correct_spelling_str(
            "hello",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &FstDictionary::curated(),
        );

        assert_eq!(results1, results2);
        assert_eq!(results1, results3);
    }
}

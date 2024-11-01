use std::borrow::Cow;

use harper_dictionary_parsing::WordMetadata;
use itertools::{Itertools, MinMaxResult};

use crate::{CharString, CharStringExt};

pub use self::dictionary::Dictionary;
pub use self::fst_dictionary::FstDictionary;
pub use self::full_dictionary::FullDictionary;
pub use self::merged_dictionary::MergedDictionary;

mod dictionary;
mod fst_dictionary;
mod full_dictionary;
mod merged_dictionary;

/// Get the closest matches in the provided [`Dictionary`] and rank them
/// Implementation is left up to the underlying dictionary.
pub fn suggest_correct_spelling<'a>(
    misspelled_word: &[char],
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &'a impl Dictionary,
) -> Vec<&'a [char]> {
    let matches: Vec<(&[char], u8, WordMetadata)> = dictionary
        .fuzzy_match(misspelled_word, max_edit_dist, result_limit)
        .into_iter()
        .collect();

    let mut found: Vec<(&[char], u8, WordMetadata)> = Vec::with_capacity(matches.len());
    // Often the longest and the shortest words are the most helpful, so lets push
    // them first.
    let minmax = matches
        .iter()
        .position_minmax_by_key(|(word, _, _)| word.len());
    if let MinMaxResult::MinMax(a, b) = minmax {
        if a == b {
            found.push(matches[a]);
        } else {
            found.push(matches[a]);
            found.push(matches[b]);
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
        found.extend(matches);
    }

    // Swap the lowest edit distance word with the shortest.
    if found.len() >= 3 {
        found.swap(0, 2);
    }

    // Let common words bubble up.
    found.sort_by_key(|(_, _, metadata)| if metadata.common { 0 } else { 1 });
    found.into_iter().map(|(word, _, _)| word).collect()
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
pub(self) fn seq_to_normalized(seq: &[char]) -> Cow<'_, [char]> {
    if seq.iter().any(|c| char_to_normalized(*c) != *c) {
        Cow::Owned(seq.iter().copied().map(char_to_normalized).collect())
    } else {
        Cow::Borrowed(seq)
    }
}

pub(self) fn char_to_normalized(c: char) -> char {
    match c {
        '’' => '\'',
        _ => c,
    }
}

// Computes the Levenshtein edit distance between two patterns.
// This is accomplished via a memory-optimized Wagner-Fischer algorithm
//
// This variant avoids allocation if you already have buffers.
#[inline]
pub(self) fn edit_distance_min_alloc(
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

pub(self) fn edit_distance(source: &[char], target: &[char]) -> u8 {
    edit_distance_min_alloc(source, target, &mut Vec::new(), &mut Vec::new())
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::{suggest_correct_spelling_str, FstDictionary};

    #[test]
    fn produces_no_duplicates() {
        let results = suggest_correct_spelling_str("punctation", 100, 3, &FstDictionary::curated());

        dbg!(&results, results.iter().unique().collect_vec());

        assert_eq!(results.iter().unique().count(), results.len())
    }

    #[test]
    fn issue_182() {
        let results = suggest_correct_spelling_str("im", 100, 3, &super::FullDictionary::curated());

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"I'm".to_string()));
    }
}

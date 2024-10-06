use std::borrow::Cow;

use itertools::{Itertools, MinMaxResult};

pub use self::dictionary::Dictionary;
pub use self::full_dictionary::FullDictionary;
pub use self::merged_dictionary::MergedDictionary;

mod dictionary;
mod fst_dictionary;
mod full_dictionary;
mod hunspell;
mod merged_dictionary;

/// Suggest a correct spelling for a given misspelled word.
/// [`misspelled_word`] is assumed to be quite small (n < 100).
/// [`max_edit_dist`] relates to an optimization that allows the search
/// algorithm to prune large portions of the search.
pub fn suggest_correct_spelling<'a>(
    misspelled_word: &[char],
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &'a impl Dictionary,
) -> Vec<&'a [char]> {
    let misspelled_word = seq_to_normalized(misspelled_word);

    let misspelled_lower: Vec<char> = misspelled_word
        .iter()
        .flat_map(|v| v.to_lowercase())
        .collect();

    // 53 is the length of the longest word.
    let mut buf_a = Vec::with_capacity(53);
    let mut buf_b = Vec::with_capacity(53);

    // The length of the shortest word to look at.
    let shortest_word_len = if misspelled_word.len() < max_edit_dist as usize {
        1
    } else {
        misspelled_word.len() - max_edit_dist as usize
    };

    // Note how we look at the biggest words first.
    let words_to_search = (shortest_word_len..misspelled_word.len() + max_edit_dist as usize)
        .rev()
        .flat_map(|len| dictionary.words_with_len_iter(len));

    let pruned_words = words_to_search.filter_map(|word| {
        let dist = edit_distance_min_alloc(&misspelled_word, word, &mut buf_a, &mut buf_b);
        let dist_lower = edit_distance_min_alloc(&misspelled_lower, word, &mut buf_a, &mut buf_b);

        if dist.min(dist_lower) <= max_edit_dist {
            Some((word, dist))
        } else {
            None
        }
    });

    // Locate the words with the lowest edit distance.
    let mut found_dist: Vec<(&[char], u8)> = Vec::with_capacity(result_limit);

    for (word, dist) in pruned_words {
        if found_dist.len() < result_limit {
            found_dist.push((word, dist));
        } else if dist < found_dist[result_limit - 1].1 {
            found_dist[result_limit - 1] = (word, dist);
        }
        found_dist.sort_by_key(|a| a.1);
    }

    // Create final, ordered list of suggestions.
    let mut found = Vec::with_capacity(found_dist.len());

    // Often the longest and the shortest words are the most helpful, so lets push
    // them first.
    let minmax = found_dist.iter().position_minmax_by_key(|a| a.0.len());
    if let MinMaxResult::MinMax(a, b) = minmax {
        if a == b {
            found.push(found_dist[a].0);
        } else {
            found.push(found_dist[a].0);
            found.push(found_dist[b].0);
        }

        // Push the rest
        found.extend(
            found_dist
                .into_iter()
                .enumerate()
                .filter(|(i, _)| *i != a && *i != b)
                .map(|v| v.1 .0),
        );
    } else {
        // Push the rest
        found.extend(found_dist.into_iter().map(|v| v.0));
    }

    // Swap the lowest edit distance word with the shortest.
    if found.len() >= 3 {
        found.swap(0, 2);
    }

    // Let common words bubble up.
    found.sort_by_key(|v| {
        if dictionary.get_word_metadata(v).common {
            0
        } else {
            1
        }
    });

    found
}

/// Convenience function over [`suggest_correct_spelling`] that does conversions
/// for you.
pub fn suggest_correct_spelling_str(
    misspelled_word: impl AsRef<str>,
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &FullDictionary,
) -> Vec<String> {
    let chars: Vec<char> = misspelled_word.as_ref().chars().collect();

    suggest_correct_spelling(&chars, result_limit, max_edit_dist, dictionary)
        .into_iter()
        .map(|word| word.iter().collect())
        .collect()
}

// Computes the Levenstein edit distance between two patterns.
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
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::{edit_distance, seq_to_normalized, suggest_correct_spelling_str};
    use crate::FullDictionary;

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
    fn simple1() {
        assert_edit_dist("kitten", "sitting", 3)
    }

    #[test]
    fn simple2() {
        assert_edit_dist("saturday", "sunday", 3)
    }

    #[test]
    fn produces_no_duplicates() {
        let results =
            suggest_correct_spelling_str("punctation", 100, 3, &FullDictionary::curated());

        dbg!(&results, results.iter().unique().collect_vec());

        assert_eq!(results.iter().unique().count(), results.len())
    }

    #[test]
    fn issue_182() {
        let results = suggest_correct_spelling_str("im", 100, 3, &FullDictionary::curated());

        dbg!(&results);

        assert!(results.iter().take(3).contains(&"I'm".to_string()));
    }
}

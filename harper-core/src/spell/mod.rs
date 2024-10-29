use itertools::{Itertools, MinMaxResult};

use crate::CharString;

pub use self::dictionary::Dictionary;
pub use self::full_dictionary::FullDictionary;
pub use self::merged_dictionary::MergedDictionary;

mod dictionary;
mod fst_dictionary;
mod full_dictionary;
mod hunspell;
mod merged_dictionary;

/// Get the closest matches in the provided [`Dictionary`] and rank them
/// Implementation is left up to the underlying dictionary.
pub fn suggest_correct_spelling<'a>(
    misspelled_word: &[char],
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &'a impl Dictionary,
) -> Vec<&'a [char]> {
    let matches: Vec<&[char]> = dictionary
        .fuzzy_match(misspelled_word, max_edit_dist, result_limit)
        .into_iter()
        .map(|r| r.0)
        .collect();

    let mut found = Vec::with_capacity(matches.len());
    // Often the longest and the shortest words are the most helpful, so lets push
    // them first.
    let minmax = matches.iter().position_minmax_by_key(|a| a.len());
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
        // Push the rest
        found.extend(matches);
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
    misspelled_word: impl Into<String>,
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &FullDictionary,
) -> Vec<String> {
    let chars: CharString = misspelled_word.into().chars().collect();
    suggest_correct_spelling(&chars, result_limit, max_edit_dist, dictionary)
        .into_iter()
        .map(|a| a.iter().collect::<String>())
        .collect()
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{spell::suggest_correct_spelling_str, FullDictionary};

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

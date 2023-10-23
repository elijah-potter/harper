use crate::words::english_words;

/// Suggest a correct spelling for a given misspelled word.
/// [misspelled_word] is assumed to be quite small (n < 100)
/// [max_edit_dist] relates to an optimization that allows the search algorithm to prune large portions of the search.
pub fn suggest_correct_spelling(
    misspelled_word: &[char],
    result_limit: usize,
    max_edit_dist: u8,
) -> Vec<&[char]> {
    let words = english_words();

    // 53 is the length of the longest word.
    let mut buf_a = Vec::with_capacity(53);
    let mut buf_b = Vec::with_capacity(53);

    let pruned_words = words
        .iter()
        .filter(|word| word.len().abs_diff(misspelled_word.len()) <= max_edit_dist as usize)
        .cloned()
        .filter_map(|word| {
            let dist = edit_distance_min_alloc(misspelled_word, word, &mut buf_a, &mut buf_b);

            if dist <= max_edit_dist {
                Some((word, dist))
            } else {
                None
            }
        });

    let mut found: Vec<(&[char], u8)> = Vec::with_capacity(result_limit);

    for (word, dist) in pruned_words {
        if found.len() < result_limit {
            found.push((word, dist));
            found.sort_by(|a, b| a.1.cmp(&b.1));
            continue;
        }

        if dist < found[result_limit - 1].1 {
            found[result_limit - 1] = (word, dist);
            found.sort_by(|a, b| a.1.cmp(&b.1));
        }
    }

    found.into_iter().map(|(word, _dist)| word).collect()
}

/// Convenience function over [suggest_correct_spelling] that does conversions for you.
pub fn suggest_correct_spelling_str(
    misspelled_word: impl AsRef<str>,
    result_limit: usize,
    max_edit_dist: u8,
) -> Vec<String> {
    let chars: Vec<char> = misspelled_word.as_ref().chars().collect();

    suggest_correct_spelling(&chars, result_limit, max_edit_dist)
        .into_iter()
        .map(|word| word.iter().collect())
        .collect()
}

// Computes the Levenstein edit distance between two patterns.
// This is accomplished via a memory-optimized Wagner-Fischer algorithm
//
// This variant avoids allocation if you already have buffers.
fn edit_distance_min_alloc(
    source: &[char],
    target: &[char],
    previous_row: &mut Vec<u8>,
    current_row: &mut Vec<u8>,
) -> u8 {
    assert!(source.len() <= 255 && target.len() <= 255);

    let row_width = source.len();
    let col_height = target.len();

    previous_row.clear();
    previous_row.extend(0u8..=row_width as u8);
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
    use super::edit_distance;

    fn assert_edit_dist(source: &str, target: &str, expected: u8) {
        let source: Vec<_> = source.chars().collect();
        let target: Vec<_> = target.chars().collect();

        let dist = edit_distance(&source, &target);
        assert_eq!(dist, expected)
    }

    #[test]
    fn simple1() {
        assert_edit_dist("kitten", "sitting", 3)
    }
    #[test]
    fn simple2() {
        assert_edit_dist("saturday", "sunday", 3)
    }
}

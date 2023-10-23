use crate::words::english_words;

pub fn suggest_correct_spelling(misspelled_word: &[char], result_limit: usize) -> Vec<&[char]> {
    let words = english_words();

    let mut buf_a = Vec::new();
    let mut buf_b = Vec::new();

    let word_dist: Vec<_> = words
        .iter()
        .map(|word| edit_distance_min_alloc(misspelled_word, word, &mut buf_a, &mut buf_b))
        .collect();

    let mut found: Vec<(&[char], u8)> = Vec::with_capacity(result_limit);
    found.extend(
        words[0..result_limit]
            .iter()
            .copied()
            .zip(word_dist[0..result_limit].iter().copied()),
    );

    found.sort_by(|a, b| a.1.cmp(&b.1));

    for (word, score) in words.iter().zip(&word_dist) {
        if *score < found[result_limit - 1].1 {
            found[result_limit - 1] = (word, *score);
            found.sort_by(|a, b| a.1.cmp(&b.1));
        }
    }

    found.into_iter().map(|(word, _dist)| word).collect()
}

pub fn suggest_correct_spelling_str(
    misspelled_word: impl AsRef<str>,
    result_limit: usize,
) -> Vec<String> {
    let chars: Vec<char> = misspelled_word.as_ref().chars().collect();

    suggest_correct_spelling(&chars, result_limit)
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

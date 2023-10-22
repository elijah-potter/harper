// Computes the Levenstein edit distance between two patterns.
// This is accomplished via the Wagner-Fischer algorithm
fn edit_distance(source: &[char], target: &[char]) -> u8 {
    assert!(source.len() <= 255 && target.len() <= 255);

    let m = source.len();
    let n = target.len();

    let mut d = create_empty_matrix(m + 1, n + 1);

    for i in 0..=m {
        d[i][0] = i as u8;
    }

    for i in 0..=n {
        d[0][i] = i as u8;
    }

    for j in 1..=n {
        for i in 1..=m {
            let cost = if source[i - 1] == target[j - 1] { 0 } else { 1 };

            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + cost);
        }
    }

    dbg!(&d);

    d[m][n]
}

// Create an empty matrix of size [m, n]
fn create_empty_matrix(m: usize, n: usize) -> Vec<Vec<u8>> {
    let mut d = Vec::with_capacity(m);

    for _ in 0..m {
        d.push(vec![0u8; n]);
    }

    d
}

#[cfg(test)]
mod tests {
    use crate::fuzzy::edit_distance;

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

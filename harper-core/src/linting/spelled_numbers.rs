use crate::{Document, Lint, LintKind, Linter, Suggestion, TokenStringExt};

/// Linter that checks to make sure small integers (< one hundred) are spelled
/// out.
#[derive(Default, Clone, Copy)]
pub struct SpelledNumbers;

impl Linter for SpelledNumbers {
    fn lint(&mut self, document: &Document) -> Vec<crate::Lint> {
        let mut lints = Vec::new();

        for number_tok in document.iter_numbers() {
            let (number, _suffix) = number_tok.kind.number().unwrap();

            if (number - number.floor()).abs() < f64::EPSILON && number <= 100. {
                lints.push(Lint {
                    span: number_tok.span,
                    lint_kind: LintKind::Readability,
                    suggestions: vec![Suggestion::ReplaceWith(
                        spell_out_number(number as u64).unwrap().chars().collect()
                    )],
                    message: "Try to spell out numbers less than a hundred.".to_string(),
                    priority: 63
                })
            }
        }

        lints
    }
}

/// Converts a number to it's spelled-out variant.
///
/// For example: 100 -> one-hundred.
///
/// Could easily be extended to support numbers greater than 110, but we don't
/// need that yet.
fn spell_out_number(num: u64) -> Option<String> {
    if num > 110 {
        return None;
    }

    Some(match num {
        0 => "zero".to_string(),
        1 => "one".to_string(),
        2 => "two".to_string(),
        3 => "three".to_string(),
        4 => "four".to_string(),
        5 => "five".to_string(),
        6 => "six".to_string(),
        7 => "seven".to_string(),
        8 => "eight".to_string(),
        9 => "nine".to_string(),
        10 => "ten".to_string(),
        11 => "eleven".to_string(),
        12 => "twelve".to_string(),
        20 => "twenty".to_string(),
        30 => "thirty".to_string(),
        40 => "forty".to_string(),
        50 => "fifty".to_string(),
        60 => "sixty".to_string(),
        70 => "seventy".to_string(),
        80 => "eighty".to_string(),
        90 => "ninety".to_string(),
        100 => "one hundred".to_string(),
        _ => {
            let parent = (num / 10) * 10;
            let child = num % 10;

            format!(
                "{} {}",
                spell_out_number(parent).unwrap(),
                spell_out_number(child).unwrap()
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::spell_out_number;

    #[test]
    fn produces_fifty_three() {
        assert_eq!(spell_out_number(53), Some("fifty three".to_string()))
    }

    #[test]
    fn produces_eighty_two() {
        assert_eq!(spell_out_number(82), Some("eighty two".to_string()))
    }
}

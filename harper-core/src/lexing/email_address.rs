use harper_data::TokenKind;
use itertools::Itertools;

use super::hostname::lex_hostname;
use super::FoundToken;

pub fn lex_email_address(source: &[char]) -> Option<FoundToken> {
    // Location of the @ sign
    let (at_loc, _) = source.iter().enumerate().rev().find(|(_, c)| **c == '@')?;

    let local_part = &source[0..at_loc];

    if !validate_local_part(local_part) {
        return None;
    }

    let domain_part_len = lex_hostname(&source[at_loc + 1..])?;

    Some(FoundToken {
        next_index: at_loc + 1 + domain_part_len,
        token: TokenKind::EmailAddress,
    })
}

/// Check to see if a given slice is a valid local part of an email address.
fn validate_local_part(mut local_part: &[char]) -> bool {
    if local_part.len() > 64 || local_part.is_empty() {
        return false;
    }

    let is_quoted =
        local_part.first().cloned() == Some('"') && local_part.last().cloned() == Some('"');

    if is_quoted && local_part.len() < 2 {
        return false;
    }

    if is_quoted {
        local_part = &local_part[1..local_part.len() - 1];
    }

    if !is_quoted {
        if !local_part.iter().cloned().all(valid_unquoted_character) {
            return false;
        }

        if local_part.first().cloned().unwrap() == '.' || local_part.last().cloned().unwrap() == '.'
        {
            return false;
        }

        for (c, n) in local_part.iter().tuple_windows() {
            if *c == '.' && *n == '.' {
                return false;
            }
        }
    } else {
        let mut iter = local_part.iter().cloned();

        while let Some(c) = iter.next() {
            if c == '\\' {
                iter.next();
                continue;
            }

            let also_valid = ['(', ')', ',', ':', ';', '<', '>', '@', '[', ']', ' '];

            if !valid_unquoted_character(c) && !also_valid.contains(&c) {
                return false;
            }
        }
    }

    true
}

/// Check if a given character is valid in an unquoted local part of an address
fn valid_unquoted_character(c: char) -> bool {
    if matches!(c,
        'A'..='Z' |
        'a'..='z' |
        '0'..='9'
    ) {
        return true;
    }

    if c > '\u{007F}' {
        return true;
    }

    let others = [
        '!', '#', '$', '%', '&', '\'', '*', '+', '-', '/', '=', '?', '^', '_', '`', '{', '|', '}',
        '~', '.',
    ];

    if others.contains(&c) {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::super::hostname::tests::example_domain_parts;
    use super::{lex_email_address, validate_local_part};

    fn example_local_parts() -> impl Iterator<Item = Vec<char>> {
        [
            r#"simple"#,
            r#"very.common"#,
            r#"x"#,
            r#"long.email-address-with-hyphens"#,
            r#"user.name+tag+sorting"#,
            r#"name/surname"#,
            r#"admin"#,
            r#"example"#,
            r#"" ""#,
            r#""john..doe""#,
            r#"mailhost!username"#,
            r#""very.(),:;<>[]\".VERY.\"very@\\ \"very\".unusual""#,
            r#"user%example.com"#,
            r#"user-"#,
            r#"postmaster"#,
            r#"postmaster"#,
            r#"_test"#,
        ]
        .into_iter()
        .map(|s| s.chars().collect())
    }

    #[test]
    fn example_local_parts_pass_validation() {
        for local in example_local_parts() {
            dbg!(local.iter().collect::<String>());
            assert!(validate_local_part(&local));
        }
    }

    #[test]
    fn test_many_example_email_addresses() {
        for local in example_local_parts() {
            for mut domain in example_domain_parts() {
                // Generate email address
                let mut address = local.clone();
                address.push('@');
                address.append(&mut domain);

                dbg!(address.iter().collect::<String>());
                let found = lex_email_address(&address).unwrap();
                assert_eq!(found.next_index, address.len());
            }
        }
    }

    /// Tests that the email parser will not throw a panic under some random
    /// situations.
    #[test]
    fn survives_random_chars() {
        let mut rng = rand::thread_rng();

        let mut buf = [' '; 128];

        for _ in 0..1 << 16 {
            rng.try_fill(&mut buf).unwrap();

            lex_email_address(&buf);
        }
    }
}

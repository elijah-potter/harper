use itertools::Itertools;

use super::FoundToken;
use crate::TokenKind;

pub fn lex_email_address(source: &[char]) -> Option<FoundToken> {
    // Location of the @ sign
    let (at_loc, _) = source.iter().enumerate().rev().find(|(_, c)| **c == '@')?;

    let local_part = &source[0..at_loc];

    if !validate_local_part(local_part) {
        return None;
    }

    let mut domain_part_len = source[at_loc + 1..]
        .iter()
        .position(|c| c.is_whitespace())
        .unwrap_or(source.len() - 1 - at_loc);

    loop {
        let domain_part = &source[at_loc + 1..at_loc + 1 + domain_part_len];

        if validate_hostname(domain_part) {
            break;
        }

        domain_part_len -= 1;
    }

    Some(FoundToken {
        next_index: at_loc + 1 + domain_part_len,
        token: TokenKind::EmailAddress
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
        '~', '.'
    ];

    if others.contains(&c) {
        return true;
    }

    false
}

/// Check if a host name is valid.
fn validate_hostname(source: &[char]) -> bool {
    if source.len() > 253 || source.is_empty() {
        return false;
    }

    for label in source.split(|c| *c == '.') {
        if label.is_empty() || label.len() > 63 {
            return false;
        }

        for c in label {
            if !matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-') {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::{lex_email_address, validate_local_part};
    use crate::lexing::email_address::validate_hostname;

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
            r#"_test"#
        ]
        .into_iter()
        .map(|s| s.chars().collect())
    }

    fn example_domain_parts() -> impl Iterator<Item = Vec<char>> {
        [
            r#"example.com"#,
            r#"example.com"#,
            r#"example.com"#,
            r#"and.subdomains.example.com"#,
            r#"example.com"#,
            r#"example.com"#,
            r#"example"#,
            r#"s.example"#,
            r#"example.org"#,
            r#"example.org"#,
            r#"example.org"#,
            r#"strange.example.com"#,
            r#"example.org"#,
            r#"example.org"# /* The existing parser intentionally doesn't support IP addresses
                              * It simply isn't worth the effort at the moment.
                              * r#"[123.123.123.123]"#,
                              * r#"[IPv6:2001:0db8:85a3:0000:0000:8a2e:0370:7334]"#,
                              * r#"[IPv6:2001:0db8:85a3:0000:0000:8a2e:0370:7334]"#, */
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
    fn example_domain_parts_pass_validation() {
        for domain in example_domain_parts() {
            dbg!(domain.iter().collect::<String>());
            assert!(validate_hostname(&domain));
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
}

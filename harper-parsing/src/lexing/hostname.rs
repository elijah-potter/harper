pub fn lex_hostname(source: &[char]) -> Option<usize> {
    let mut passed_chars = 0;

    for label in source.split(|c| *c == '.') {
        for c in label {
            passed_chars += 1;
            if !matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-') {
                return Some(passed_chars - 1);
            }
        }

        passed_chars += 1;
    }

    if passed_chars == 0 {
        None
    } else {
        Some(passed_chars - 1)
    }
}

#[cfg(test)]
pub mod tests {
    use super::lex_hostname;

    pub fn example_domain_parts() -> impl Iterator<Item = Vec<char>> {
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
            r#"example.org"#,
        ]
        .into_iter()
        .map(|s| s.chars().collect())
    }

    #[test]
    fn can_parse_example_hostnames() {
        for domain in example_domain_parts() {
            dbg!(domain.iter().collect::<String>());
            assert_eq!(lex_hostname(&domain), Some(domain.len()));
        }
    }
}

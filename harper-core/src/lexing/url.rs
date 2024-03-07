/// This module implements parsing of URIs.
/// See RFC 1738 for more information.
use super::{hostname::lex_hostname, FoundToken};
use crate::TokenKind;

pub fn lex_url(source: &[char]) -> Option<FoundToken> {
    let sep = source.iter().position(|c| *c == ':')?;

    if !validate_scheme(&source[0..sep]) {
        return None;
    }

    let url_end = lex_ip_schemepart(&source[sep + 1..])?;

    Some(FoundToken {
        next_index: url_end + sep + 1,
        token: TokenKind::Url,
    })
}

/// Checks whether a given char string is a valid "scheme" part of a URI.
fn validate_scheme(source: &[char]) -> bool {
    source.iter().all(|c: &char| valid_scheme_char(*c))
}

fn lex_ip_schemepart(source: &[char]) -> Option<usize> {
    if !matches!(source, ['/', '/', ..]) {
        return None;
    }

    let rest = &source[2..];

    let login_end = lex_login(rest).unwrap_or(0);

    let mut cursor = login_end;

    // Parse endpoint path
    while cursor != rest.len() {
        dbg!(&rest[cursor..]);

        if rest[cursor] != '/' {
            break;
        }

        cursor += 1;

        let next_idx = lex_xchar_string(&rest[cursor..]);

        if next_idx == 0 {
            break;
        }

        cursor += next_idx;
    }

    Some(cursor + 2)
}

fn lex_login(source: &[char]) -> Option<usize> {
    let hostport_start = if let Some(cred_end) = source.iter().position(|c| *c == '@') {
        if let Some(pass_beg) = source[0..cred_end].iter().position(|c| *c == ':') {
            if !is_uchar_plus_string(&source[pass_beg + 1..cred_end]) {
                return None;
            }
        }

        // Check username
        if !is_uchar_plus_string(&source[0..cred_end]) {
            return None;
        }

        cred_end + 1
    } else {
        0
    };

    let hostport_source = &source[hostport_start..];

    let hostport_end = lex_hostport(hostport_source)?;

    Some(hostport_start + hostport_end)
}

fn lex_hostport(source: &[char]) -> Option<usize> {
    let hostname_end = lex_hostname(source)?;

    dbg!(&source[..hostname_end]);

    if source.get(hostname_end) == Some(&':') {
        Some(
            source
                .iter()
                .enumerate()
                .find(|(_, c)| !{
                    let c = **c;
                    c.is_ascii_digit()
                })
                .map(|(i, _)| i)
                .unwrap_or(source.len()),
        )
    } else {
        Some(hostname_end)
    }
}

fn valid_scheme_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c.is_ascii_digit() || matches!(c, '.' | '-' | '+')
}

fn is_reserved(c: char) -> bool {
    matches!(c, ';' | '/' | '?' | ':' | '@' | '&' | '=')
}

fn is_safe(c: char) -> bool {
    matches!(c, '$' | '-' | '_' | '.' | '+')
}

fn is_extra(c: char) -> bool {
    matches!(c, '!' | '*' | '\'' | '(' | ')' | ',')
}

fn is_unreserved(c: char) -> bool {
    c.is_ascii_alphabetic() || c.is_ascii_digit() || is_safe(c) || is_extra(c)
}

fn is_hex(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, 'A'..='F' | 'a'..='f')
}

/// Lex an escaped hex code, returning the subsequent index
fn lex_escaped(source: &[char]) -> Option<usize> {
    if source.len() < 3 {
        return None;
    }

    if source[0] == '%' && is_hex(source[1]) && is_hex(source[2]) {
        Some(3)
    } else {
        None
    }
}

fn lex_xchar_string(source: &[char]) -> usize {
    let mut cursor = 0;

    while cursor != source.len() {
        let Some(next) = lex_xchar(&source[cursor..]) else {
            break;
        };

        cursor += next;
    }

    cursor
}

fn is_xchar_string(source: &[char]) -> bool {
    lex_xchar_string(source) == source.len()
}

/// Used for passwords and usernames
fn is_uchar_plus_string(source: &[char]) -> bool {
    let mut cursor = 0;

    while cursor != source.len() {
        if matches!(source[cursor], ';' | '?' | '&' | '=') {
            cursor += 1;
            continue;
        }

        let Some(next) = lex_uchar(&source[cursor..]) else {
            return false;
        };

        cursor += next;
    }

    true
}

fn lex_xchar(source: &[char]) -> Option<usize> {
    if is_reserved(source[0]) {
        return Some(1);
    }

    lex_uchar(source)
}

fn lex_uchar(source: &[char]) -> Option<usize> {
    if is_unreserved(source[0]) {
        return Some(1);
    }

    lex_escaped(source)
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::lex_url;

    fn assert_consumes_full(url: &str) {
        assert_consumes_part(url, url.len());
    }

    fn assert_consumes_part(url: &str, len: usize) {
        let url = url.chars().collect::<Vec<_>>();

        assert_eq!(lex_url(&url).unwrap().next_index, len);
    }

    #[test]
    fn consumes_google() {
        assert_consumes_full("https://google.com")
    }

    #[test]
    fn consumes_wikipedia() {
        assert_consumes_full("https://wikipedia.com")
    }

    #[test]
    fn consumes_youtube() {
        assert_consumes_full("https://youtube.com")
    }

    #[test]
    fn consumes_youtube_not_garbage() {
        assert_consumes_part("https://youtube.com aklsjdha", 19);
    }

    #[test]
    fn consumes_with_path() {
        assert_consumes_full("https://elijahpotter.dev/articles/quantifying_hope_on_a_global_scale")
    }

    /// Tests that the URL parser will not throw a panic under some random situations.
    #[test]
    fn survives_random_chars() {
        let mut rng = rand::thread_rng();

        let mut buf = [' '; 128];

        for _ in 0..1 << 16 {
            rng.try_fill(&mut buf).unwrap();

            lex_url(&buf);
        }
    }
}

use crate::{Dictionary, Document, TokenKind};

/// Check if the contents of the document are likely intended to represent
/// English.
pub fn is_likely_english(doc: &Document, dict: &impl Dictionary) -> bool {
    let mut total_words = 0;
    let mut valid_words = 0;
    let mut punctuation = 0;

    for token in doc.tokens() {
        match token.kind {
            TokenKind::Word => {
                total_words += 1;

                let word_content = doc.get_span_content(token.span);
                if dict.contains_word(word_content) {
                    valid_words += 1;
                }
            }
            TokenKind::Punctuation(_) => punctuation += 1,
            _ => ()
        }
    }

    dbg!(total_words);
    dbg!(valid_words);
    dbg!(punctuation);

    if (punctuation as f32 * 1.25) > valid_words as f32 {
        return false;
    }

    if (valid_words as f64 / total_words as f64) < 0.4 {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::is_likely_english;
    use crate::{Document, FullDictionary};

    fn assert_not_english(source: &'static str) {
        let dict = FullDictionary::create_from_curated();
        let doc = Document::new_plain_english(source);
        let is_likely_english = is_likely_english(&doc, &dict);
        dbg!(source);
        assert!(!is_likely_english);
    }

    fn assert_english(source: &'static str) {
        let dict = FullDictionary::create_from_curated();
        let doc = Document::new_plain_english(source);
        let is_likely_english = is_likely_english(&doc, &dict);
        dbg!(source);
        assert!(is_likely_english);
    }

    #[test]
    fn detects_spanish() {
        assert_not_english("Esto es español. Harper no debería marcarlo como inglés.");
    }

    #[test]
    fn detects_french() {
        assert_not_english(
            "C'est du français. Il ne devrait pas être marqué comme anglais par Harper."
        );
    }

    #[test]
    fn detects_shebang() {
        assert_not_english("#! /bin/bash");
        assert_not_english("#! /usr/bin/fish");
    }

    #[test]
    fn detects_short_english() {
        assert_english("This is English!");
    }

    #[test]
    fn detects_english() {
        assert_english("This is perfectly valid English, evn if it has a cople typos.")
    }

    #[test]
    fn detects_expressive_english() {
        assert_english("Look above! That is real English! So is this: bippity bop!")
    }

    /// Useful for detecting commented-out code.
    #[test]
    fn detects_python_fib() {
        assert_not_english(
            r#"
def fibIter(n):
    if n < 2:
        return n
    fibPrev = 1
    fib = 1
    for _ in range(2, n):
        fibPrev, fib = fib, fib + fibPrev
    return fib
        "#
        );
    }
}

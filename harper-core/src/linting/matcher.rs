use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{CharString, Document, Punctuation, Span, Token, TokenKind, WordMetadata};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct PatternToken {
    /// The general variant of the token.
    /// The inner data of the [`TokenKind`] should be replaced with the default
    /// value.
    kind: TokenKind,
    content: Option<CharString>,
}

impl PatternToken {
    fn from_token(token: Token, document: &Document) -> Self {
        if token.kind.is_word() {
            Self {
                kind: token.kind.with_default_data(),
                content: Some(document.get_span_content(token.span).into()),
            }
        } else {
            Self {
                kind: token.kind,
                content: None,
            }
        }
    }
}

macro_rules! vecword {
    ($lit:literal) => {
        $lit.chars().collect()
    };
}

macro_rules! pt {
    ($str:literal) => {
        PatternToken {
            kind: TokenKind::Word(WordMetadata::default()),
            content: Some($str.chars().collect()),
        }
    };
    (Word) => {
        PatternToken {
            kind: TokenKind::Word,
            content: None,
        }
    };
    (Period) => {
        PatternToken {
            kind: TokenKind::Punctuation(Punctuation::Period),
            content: None,
        }
    };
    (Hyphen) => {
        PatternToken {
            kind: TokenKind::Punctuation(Punctuation::Hyphen),
            content: None,
        }
    };
    (Space) => {
        PatternToken {
            kind: TokenKind::Space(1),
            content: None,
        }
    };
    ( $($($str:literal),* => $repl:literal),*) => {
        vec![
            $(
                {
                    let mut rule = Rule {
                        pattern: vec![$(
                            pt!($str),
                            pt!(Space),
                        )*],
                        replace_with: $repl.chars().collect()
                    };

                    if rule.pattern.len() > 0{
                        rule.pattern.pop();
                    }

                    rule
                },
            )*
        ]
    };
}

struct Rule {
    pattern: Vec<PatternToken>,
    replace_with: Vec<char>,
}

/// A linter that uses a variety of curated pattern matches to find and fix
/// common grammatical issues.
pub struct Matcher {
    triggers: Vec<Rule>,
}

impl Matcher {
    pub fn new() -> Self {
        // This match list needs to be automatically expanded instead of explicitly
        // defined like it is now.
        let mut triggers = pt! {
            "spacial","attention" => "special attention",
            "wellbeing" => "well-being",
            "hashtable" => "hash table",
            "hashmap" => "hash map",
            "CCP" => "Chinese Communist Party",
            "dep" => "dependency",
            "deps" => "dependencies",
            "off","the","cuff" => "off-the-cuff",
            "an","in" => "and in",
            "my","self" => "myself",
            "human","live" => "human life",
            "eight","grade" => "eighth grade",
            "and","also" => "and",
            "todo" => "to-do",
            "To-Do" => "To-do",
            "performing","this" => "perform this",
            "united nations" => "United Nations",
            "mins" => "minutes",
            "min" => "minute",
            "secs" => "seconds",
            "sec" => "second",
            "hrs" => "hours",
            "hr" => "hour",
            "w/o" => "without",
            "w/" => "with",
            "wordlist" => "word list" ,
            "the","challenged" => "that challenged",
            "stdin" => "standard input",
            "stdout" => "standard output",
            "no","to" => "not to",
            "No","to" => "not to",
            "ngram" => "n-gram",
            "grammer" => "grammar",
            "There","fore" => "Therefore",
            "south","America" => "South America",
            "South","america" => "South America",
            "south","america" => "South America",
            "North","america" => "North America",
            "north","America" => "North America",
            "north","america" => "North America",
            "fatal","outcome" => "death",
            "geiger","counter" => "Geiger counter",
            "Geiger","Counter" => "Geiger counter",
            "veterans","day" => "Veterans Day",
            "presidents","day" => "Presidents' Day",
            "president's","day" => "Presidents' Day",
            "valentines","day" => "Valentine's Day",
            "world","war","2" => "World War II",
            "World","war","ii" => "World War II",
            "world","War","ii" => "World War II",
            "World","War","Ii" => "World War II",
            "World","War","iI" => "World War II",
            "black","sea" => "Black Sea",
            "I","a","m" => "I am",
            "We","a","re" => "We are",
            "The","re" => "There",
            "my","french" => "my French",
            "It","cam" => "It can",
            "can","be","seem" => "can be seen",
            "mu","house" => "my house",
            "kid","regards" => "kind regards",
            "miss","understand" => "misunderstand",
            "miss","use" => "misuse",
            "miss","used" => "misused",
            "bee","there" => "been there",
            "want","be" => "won't be",
            "more","then" => "more than",
            "gong","to" => "going to",
            "then","others" => "than others",
            "Then","others" => "than others",
            "then","before" => "than before",
            "Then","before" => "than before",
            "then","last","week" => "than last week",
            "then","her" => "than her",
            "then","hers" => "than hers",
            "then","him" => "than him",
            "then","his" => "than his",
            "simply","grammatical" => "simple grammatical",
            "you","r" => "your",
            "you","re" => "you're",
            "that","s" => "that's",
            "That","s" => "That's",
            "that","s" => "that is",
            "That","s" => "that is",
            "ms" => "milliseconds",
            "t","he" => "the",
            "the","hing" => "the thing",
            "The","hing" => "The thing",
            "need","helps" => "need help",
            "all","though" => "although",
            "All","though" => "although",
            "al","though" => "although",
            "Al","though" => "although",
            "an","this" => "and this",
            "break","up" => "break-up",
            "case", "sensitive" => "case-sensitive",
            "bare", "foot" => "barefoot",
            "air", "port" => "airport",
            "any", "body" => "anybody",
            "every", "body" => "everybody",
            "no", "body" => "nobody",
            "some", "body" => "somebody",
            "any", "one" => "anyone",
            "every", "one" => "everyone",
            "some", "one" => "someone",
            "any", "thing" => "anything",
            "every", "thing" => "everything",
            "no", "thing" => "nothing",
            "some", "thing" => "something",
            "any", "where" => "anywhere",
            "every", "where" => "everywhere",
            "no", "where" => "nowhere",
            "some", "where" => "somewhere",
            "baby", "sit" => "babysit",
            "back", "ground" => "background",
            "bare", "foot" => "barefoot",
            "base", "ball" => "baseball",
            "basket", "ball" => "basketball",
            "foot", "ball" => "football",
            "bath", "room" => "bathroom",
            "bed", "room" => "bedroom",
            "black", "berry" => "blackberry",
            "blue", "berry" => "blueberry",
            "break", "fast" => "breakfast",
            "can", "not" => "cannot",
            "check", "out" => "checkout",
            "cow", "boy" => "cowboy",
            "day", "light" => "daylight",
            "desk", "top" => "desktop",
            "finger", "print" => "fingerprint",
            "fire", "fly" => "firefly",
            "fore", "ver" => "forever",
            "gentle", "man" => "gentleman",
            "grand", "mother" => "grandmother",
            "grand", "father" => "grandfather",
            "grand", "daughter" => "granddaughter",
            "grape", "fruit" => "grapefruit",
            "grass", "hopper" => "grasshopper",
            "head", "quarters" => "headquarters",
            "hand", "shake" => "handshake",
            "in", "side" => "inside",
            "key", "board" => "keyboard",
            "lip", "stick" => "lipstick",
            "mail", "box" => "mailbox",
            "never", "theless" => "nevertheless",
            "none", "theless" => "nonetheless",
            "note", "book" => "notebook",
            "ou", "tside" => "outside",
            "pay", "day" => "payday",
            "rail", "road" => "railroad",
            "rain", "bow" => "rainbow",
            "rain", "coat" => "raincoat",
            "skate", "board" => "skateboard",
            "smart", "phone" => "smartphone",
            "snow", "ball" => "snowball",
            "some", "times" => "sometimes",
            "sun", "flower" => "sunflower",
            "tooth", "brush" => "toothbrush",
            "turn", "table" => "turntable",
            "under", "cover" => "undercover",
            "up", "stream" => "upstream",
            "water", "fall" => "waterfall",
            "water", "melon" => "watermelon",
            "wee", "kend" => "weekend",
            "with", "in" => "within",
            "with", "out" => "without",
            "TreeSitter" => "Tree-sitter",
            "Treesitter" => "Tree-sitter",
            "Tree", "sitter" => "Tree-sitter",
            "all", "of", "the" => "all the",
            "an", "other" => "another",
            "not", "longer" => "no longer",
            "to", "towards" => "towards",
            "though", "process" => "thought process",
            "the", "this" => "that this"
        };

        // TODO: Improve the description for this lint specifically.
        // We need to be more explicit that we are replacing with an Em dash
        triggers.push(Rule {
            pattern: vec![pt!(Hyphen), pt!(Hyphen), pt!(Hyphen)],
            replace_with: vecword!("—"),
        });

        // Same goes for this En dash
        triggers.push(Rule {
            pattern: vec![pt!(Hyphen), pt!(Hyphen)],
            replace_with: vecword!("–"),
        });

        triggers.push(Rule {
            pattern: vec![pt!("L"), pt!(Period), pt!("L"), pt!(Period), pt!("M")],
            replace_with: vecword!("large language model"),
        });

        triggers.push(Rule {
            pattern: vec![
                pt!("L"),
                pt!(Period),
                pt!("L"),
                pt!(Period),
                pt!("M"),
                pt!(Period),
            ],
            replace_with: vecword!("large language model"),
        });

        Self { triggers }
    }
}

impl Default for Matcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for Matcher {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        let mut match_tokens = Vec::new();

        for (index, _) in document.tokens().enumerate() {
            for trigger in &self.triggers {
                match_tokens.clear();

                for (p_index, pattern) in trigger.pattern.iter().enumerate() {
                    let Some(token) = document.get_token(index + p_index) else {
                        break;
                    };

                    let t_pattern = PatternToken::from_token(token, document);

                    if t_pattern != *pattern {
                        break;
                    }

                    match_tokens.push(token);
                }

                if match_tokens.len() == trigger.pattern.len() && !match_tokens.is_empty() {
                    let span = Span::new(
                        match_tokens.first().unwrap().span.start,
                        match_tokens.last().unwrap().span.end,
                    );

                    lints.push(Lint {
                        span,
                        lint_kind: LintKind::Miscellaneous,
                        suggestions: vec![Suggestion::ReplaceWith(trigger.replace_with.to_owned())],
                        message: format!(
                            "Did you mean “{}”?",
                            trigger.replace_with.iter().collect::<String>()
                        ),
                        priority: 15,
                    })
                }
            }
        }

        lints
    }
}

#[cfg(test)]
mod tests {
    use super::{Linter, Matcher};
    use crate::Document;

    #[test]
    fn matches_therefore() {
        let document = Document::new_plain_english_curated("There fore.");
        let mut matcher = Matcher::new();
        let lints = matcher.lint(&document);
        assert_eq!(lints.len(), 1);
    }
}

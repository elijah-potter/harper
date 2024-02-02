use crate::{
    spell::DictWord, Document, Lint, LintKind, Linter, Punctuation, Span, Suggestion, Token,
    TokenKind,
};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct PatternToken {
    kind: TokenKind,
    content: Option<DictWord>,
}

impl PatternToken {
    fn from_token(token: Token, document: &Document) -> Self {
        if token.kind.is_word() {
            Self {
                kind: token.kind,
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
            kind: TokenKind::Word,
            content: Some($str.chars().collect()),
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

/// A linter that uses a variety of curated pattern matches to find and fix common
/// grammatical issues.
pub struct Matcher {
    triggers: Vec<Rule>,
}

impl Matcher {
    pub fn new() -> Self {
        let mut triggers = pt! {
            "the","challenged" => "that challenged",
            "stdin" => "standard input",
            "stdout" => "standard output",
            "your","doing" => "you're doing",
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
            "that","s" => "that's",
            "That","s" => "That's",
            "that","s" => "that is",
            "That","s" => "that is",
            "ms" => "milliseconds",
            "LLM" => "large language model",
            "LLMs" => "large language models",
            "t","he" => "the",
            "the","hing" => "the thing",
            "The","hing" => "The thing",
            "need","helps" => "need help",
            "all","though" => "although",
            "All","though" => "although",
            "al","though" => "although",
            "Al","though" => "although"
        };

        triggers.push(Rule {
            pattern: vec![pt!("break"), pt!(Hyphen), pt!("up")],
            replace_with: vecword!("break-up"),
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

        for (index, _) in document.tokens().enumerate() {
            for trigger in &self.triggers {
                let mut match_tokens = Vec::new();

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
    use crate::{Document, Linter};

    use super::Matcher;

    #[test]
    fn matches_therefore() {
        let document = Document::new_plain_english("There fore.");
        let mut matcher = Matcher::new();
        let lints = matcher.lint(&document);
        assert!(lints.len() == 1)
    }
}

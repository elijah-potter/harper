use super::{
    lint::Linter, long_sentences, repeated_words, sentence_capitalization, spell_check,
    unclosed_quotes, wrong_quotes,
};
use paste::paste;

use super::{
    long_sentences::long_sentences, repeated_words::repeated_words,
    sentence_capitalization::sentence_capitalization, spell_check::spell_check,
    unclosed_quotes::unclosed_quotes, wrong_quotes::wrong_quotes,
};

#[derive(Debug, Clone)]
pub struct LintSet {
    pub(super) linters: Vec<Linter>,
}

impl LintSet {
    pub fn new() -> Self {
        Self {
            linters: Vec::new(),
        }
    }
}

impl Default for LintSet {
    fn default() -> Self {
        Self::new()
            .with_spell_check()
            .with_repeated_words()
            .with_long_sentences()
            .with_unclosed_quotes()
            .with_sentence_capitalization()
    }
}

macro_rules! create_builder {
    ($($linter:ident),*) => {
        impl LintSet {
            pub fn add_all(&mut self) -> &mut Self {
                self.linters.extend_from_slice(&[
                    $(
                        $linter
                    ),*
                ]);

                self
            }

            paste! {
                $(
                    #[doc = "Modifies self, adding the `" $linter "` linter to the set."]
                    pub fn [<add_$linter>](&mut self) -> &mut Self{
                        self.linters.push($linter);
                        self
                    }
                )*
            }

            paste! {
                $(
                    #[doc = "Consumes self, adding the `" $linter "` linter to the set."]
                    pub fn [<with_$linter>](mut self) -> Self{
                        self.linters.push($linter);
                        self
                    }
                )*
            }
        }
    };
}

create_builder!(
    spell_check,
    sentence_capitalization,
    unclosed_quotes,
    wrong_quotes,
    repeated_words,
    long_sentences
);

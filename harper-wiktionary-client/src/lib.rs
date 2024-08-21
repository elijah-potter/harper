use std::collections::HashMap;

use harper_core::{WordKind, WordMetadata};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct WiktionaryClient {
    http_client: reqwest::Client,
}

impl WiktionaryClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_definition(
        &self,
        word: &str,
    ) -> anyhow::Result<Option<HashMap<String, Vec<Definition>>>> {
        let definition_endpoint = "https://en.wiktionary.org/api/rest_v1/page/definition/";

        let resp = self
            .http_client
            .get(format!(
                "{definition_endpoint}{}",
                urlencoding::encode(word)
            ))
            .send()
            .await?;

        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        Ok(resp.json().await?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Definition {
    #[serde(rename(serialize = "partOfSpeech", deserialize = "partOfSpeech"))]
    part_of_speech: String,
    language: String,
}

impl Definition {
    /// Converts information from a Wiktionary [`Definition`] to Harper [`WordMetadata`].
    pub fn to_word_metadata(&self) -> WordMetadata {
        let kind = match self.part_of_speech.as_str() {
            "Proper noun" => Some(WordKind::Noun {
                is_proper: Some(true),
                is_plural: None,
            }),
            "Noun" => Some(WordKind::Noun {
                is_proper: None,
                is_plural: None,
            }),
            "Adjective" => Some(WordKind::Adjective),
            "Verb" => Some(WordKind::Verb),
            _ => None,
        };

        WordMetadata {
            kind,
            tense: None,
            possessive: None,
        }
    }
}

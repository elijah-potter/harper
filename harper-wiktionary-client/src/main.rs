use harper_core::{Dictionary, FullDictionary};
use harper_wiktionary_client::WiktionaryClient;

#[tokio::main]
async fn main() {
    let curated = FullDictionary::curated();
    let client = WiktionaryClient::default();

    for word in curated.words_iter().skip(10000) {
        let word = word.iter().collect::<String>();
        dbg!(word.as_str());

        let Some(def) = client.get_definition(word.as_str()).await.unwrap() else {
            continue;
        };
        dbg!(&def.get("en").unwrap());
    }
}

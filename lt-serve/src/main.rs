use lt_core::{lex_to_end, suggest_correct_spelling_str, Dictionary, Token};
use std::net::SocketAddr;
use tracing::info;

use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/parse", get(parse_text))
        .route("/spellcheck", get(spellcheck));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    info!("Hello world");

    "Hello, World!"
}

async fn parse_text(Query(payload): Query<ParseRequest>) -> (StatusCode, String) {
    info!("Parse request for: {:?}", payload.text);

    let chars: Vec<_> = payload
        .text
        .replace('\u{a0}', " ")
        .replace("<br>", "\n")
        .chars()
        .collect();

    let lexed = lex_to_end(&chars);

    let mut html = "
        <style>
	        mark {
	        	background-color: none;
	        }

	        .word {
	        	border-bottom: 1px solid black;
	        }
        </style>
        "
    .to_string();

    for token in lexed {
        let chunk = match token.kind {
            lt_core::TokenKind::Word => {
                format!(
                    r#"<mark class="word">{}</mark>"#,
                    token.span.get_content_string(&chars)
                )
            }
            lt_core::TokenKind::Punctuation(_) => {
                format!(
                    r#"<mark class="punct">{}</mark>"#,
                    token.span.get_content_string(&chars)
                )
            }
            lt_core::TokenKind::Number(_) => {
                format!(
                    r#"<mark class="number">{}</mark>"#,
                    token.span.get_content_string(&chars)
                )
            }
            lt_core::TokenKind::Space(count) => "\u{a0}".repeat(count),
            lt_core::TokenKind::Newline(count) => "<br>".repeat(count),
        };

        html.push_str(&chunk);
    }

    (StatusCode::ACCEPTED, html)
}

#[derive(Deserialize)]
struct ParseRequest {
    pub text: String,
}

async fn spellcheck(
    Query(payload): Query<SpellcheckRequest>,
) -> (StatusCode, Json<SpellcheckResponse>) {
    info!("Spellcheck request for {:?}", payload.word);

    let dictionary = Dictionary::create_from_static();

    let results = suggest_correct_spelling_str(payload.word, 5, 3, &dictionary);

    (
        StatusCode::ACCEPTED,
        axum::Json(SpellcheckResponse { words: results }),
    )
}

#[derive(Deserialize)]
struct SpellcheckRequest {
    pub word: String,
}

#[derive(Serialize)]
struct SpellcheckResponse {
    pub words: Vec<String>,
}

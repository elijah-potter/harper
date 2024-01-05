#![allow(dead_code)]

use lt_core::{all_linters, Document, FatToken, Lint, Span, Suggestion};
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
        .route("/lint", get(lint))
        .route("/apply", get(apply_suggestion));

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

async fn parse_text(Query(payload): Query<ParseRequest>) -> (StatusCode, Json<ParseResponse>) {
    let text = payload.text;

    let document = Document::new(&text);
    let tokens: Vec<_> = document.fat_tokens().collect();

    (StatusCode::ACCEPTED, Json(ParseResponse { tokens }))
}

#[derive(Deserialize)]
struct ParseRequest {
    pub text: String,
}

#[derive(Serialize)]
struct ParseResponse {
    pub tokens: Vec<FatToken>,
}

async fn lint(Query(payload): Query<LintRequest>) -> (StatusCode, Json<LintResponse>) {
    let text = payload.text;

    let document = Document::new(&text);

    let lints = all_linters(&document);

    (StatusCode::ACCEPTED, Json(LintResponse { lints }))
}

#[derive(Deserialize)]
struct LintRequest {
    pub text: String,
}

#[derive(Serialize)]
struct LintResponse {
    pub lints: Vec<Lint>,
}

async fn apply_suggestion(
    Query(payload): Query<ApplySuggestionRequest>,
) -> (StatusCode, Json<ApplySuggestionResponse>) {
    let text = payload.text;

    let Ok(SuggestionData { suggestion, span }) = serde_json::from_str(&payload.data) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApplySuggestionResponse {
                text: "".to_string(),
            }),
        );
    };

    let mut document = Document::new(&text);

    document.apply_suggestion(&suggestion, span);

    (
        StatusCode::ACCEPTED,
        Json(ApplySuggestionResponse {
            text: document.get_full_string(),
        }),
    )
}

#[derive(Deserialize)]
struct ApplySuggestionRequest {
    pub text: String,
    pub data: String,
}

#[derive(Deserialize)]
struct SuggestionData {
    suggestion: Suggestion,
    span: Span,
}

#[derive(Serialize)]
struct ApplySuggestionResponse {
    pub text: String,
}

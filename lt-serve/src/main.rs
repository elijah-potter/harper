#![allow(dead_code)]

use lt_core::{all_linters, Dictionary, Document, FatToken, Lint, Span, Suggestion};
use std::net::SocketAddr;
use tokio::time::Instant;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

use axum::{
    body::Body,
    http::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let app = Router::new()
        .route("/", post(root))
        .route("/parse", post(parse_text))
        .route("/lint", post(lint))
        .route("/apply", post(apply_suggestion))
        .layer(middleware::from_fn(timing_middleware));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn timing_middleware(request: Request<Body>, next: Next<Body>) -> Response {
    let uri = request.uri().clone();

    let start = Instant::now();

    let res = next.run(request).await;

    let end = Instant::now();

    let diff = end - start;

    info!(
        "Took {} ms to process request at endpoint: {}",
        diff.as_millis(),
        uri.path(),
    );

    res
}

async fn root() -> &'static str {
    info!("Hello world");

    "Hello, World!"
}

async fn parse_text(Json(payload): Json<ParseRequest>) -> (StatusCode, Json<ParseResponse>) {
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

async fn lint(Json(payload): Json<LintRequest>) -> (StatusCode, Json<LintResponse>) {
    let text = payload.text;

    let dictionary = Dictionary::new();
    let document = Document::new(&text);

    let lints = all_linters(&document, dictionary);

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
    Json(payload): Json<ApplySuggestionRequest>,
) -> (StatusCode, Json<ApplySuggestionResponse>) {
    let text = payload.text;
    let mut document = Document::new(&text);
    document.apply_suggestion(&payload.suggestion, payload.span);

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
    pub suggestion: Suggestion,
    pub span: Span,
}

#[derive(Serialize)]
struct ApplySuggestionResponse {
    pub text: String,
}

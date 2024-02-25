#![allow(dead_code)]

use std::net::SocketAddr;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::post;
use axum::{Json, Router};
use harper_core::{Document, FatToken, FullDictionary, Lint, LintSet, Linter, Span, Suggestion};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

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
    info!("Handling request at endpoint: {}", request.uri().path());

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

    let document = Document::new_markdown(&text);
    let tokens: Vec<_> = document.fat_tokens().collect();

    (StatusCode::ACCEPTED, Json(ParseResponse { tokens }))
}

#[derive(Deserialize)]
struct ParseRequest {
    pub text: String
}

#[derive(Serialize)]
struct ParseResponse {
    pub tokens: Vec<FatToken>
}

async fn lint(Json(payload): Json<LintRequest>) -> (StatusCode, Json<LintResponse>) {
    let text = payload.text;

    let document = Document::new_markdown(&text);

    let dictionary = FullDictionary::create_from_curated();
    let mut linter = LintSet::new().with_standard(dictionary);
    let lints = linter.lint(&document);

    (StatusCode::ACCEPTED, Json(LintResponse { lints }))
}

#[derive(Deserialize)]
struct LintRequest {
    pub text: String
}

#[derive(Serialize)]
struct LintResponse {
    pub lints: Vec<Lint>
}

async fn apply_suggestion(
    Json(payload): Json<ApplySuggestionRequest>
) -> (StatusCode, Json<ApplySuggestionResponse>) {
    let text = payload.text;
    let mut document = Document::new_markdown(&text);
    document.apply_suggestion(&payload.suggestion, payload.span);

    (
        StatusCode::ACCEPTED,
        Json(ApplySuggestionResponse {
            text: document.get_full_string()
        })
    )
}

#[derive(Deserialize)]
struct ApplySuggestionRequest {
    pub text: String,
    pub suggestion: Suggestion,
    pub span: Span
}

#[derive(Serialize)]
struct ApplySuggestionResponse {
    pub text: String
}

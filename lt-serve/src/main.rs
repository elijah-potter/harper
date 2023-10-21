use lt_core::{lex_to_end, Token};
use std::net::SocketAddr;
use tracing::info;

use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/parse", get(parse_text));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    info!("Hello world");

    "Hello, World!"
}

async fn parse_text(Query(payload): Query<ParseRequest>) -> (StatusCode, Json<Vec<Token>>) {
    info!("Parse request for: \"{}\"", payload.text);

    let chars: Vec<_> = payload.text.chars().collect();

    let lexed = lex_to_end(&chars);

    (StatusCode::ACCEPTED, Json(lexed))
}

#[derive(Deserialize)]
struct ParseRequest {
    pub text: String,
}

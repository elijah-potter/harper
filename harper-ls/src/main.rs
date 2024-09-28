#![doc = include_str!("../README.md")]

use std::io::stderr;

use config::Config;
use tokio::net::TcpListener;
mod backend;
mod config;
mod diagnostics;
mod dictionary_io;
mod document_state;
mod git_commit_parser;
mod pos_conv;

use backend::Backend;
use clap::Parser;
use tower_lsp::{LspService, Server};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static DEFAULT_ADDRESS: &str = "127.0.0.1:4000";

/// Start a language server to provide grammar checking inside of developer
/// environments.
///
/// Will listen on 127.0.0.1:4000 by default.
#[derive(Debug, Parser)]
struct Args {
    /// Set to listen on standard input / output rather than TCP.
    #[arg(short, long, default_value_t = false)]
    stdio: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .map_writer(move |_| stderr)
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    let config = Config::default();

    let (service, socket) = LspService::new(|client| Backend::new(client, config));

    if args.stdio {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        Server::new(stdin, stdout, socket).serve(service).await;
    } else {
        let listener = TcpListener::bind(DEFAULT_ADDRESS).await.unwrap();
        println!("Listening on {}", DEFAULT_ADDRESS);
        let (stream, _) = listener.accept().await.unwrap();
        let (read, write) = tokio::io::split(stream);
        Server::new(read, write, socket).serve(service).await;
    }

    Ok(())
}

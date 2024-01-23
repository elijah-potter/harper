use tokio::net::TcpListener;
mod backend;
mod diagnostics;
mod pos_conv;

use backend::Backend;
use clap::Parser;
use tower_lsp::{LspService, Server};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    stdio: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (service, socket) = LspService::new(Backend::new);

    if args.stdio {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        Server::new(stdin, stdout, socket).serve(service).await;
    } else {
        let listener = TcpListener::bind("127.0.0.1:4000").await.unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let (read, write) = tokio::io::split(stream);
        Server::new(read, write, socket).serve(service).await;
    }
}

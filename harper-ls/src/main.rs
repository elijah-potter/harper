use config::Config;
use tokio::{fs, net::TcpListener};
mod backend;
mod config;
mod diagnostics;
mod dictionary_io;
mod pos_conv;
mod tree_sitter_parser;

use backend::Backend;
use clap::Parser;
use tower_lsp::{LspService, Server};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    stdio: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let config = Config::default();

    // Make sure this is available.
    fs::create_dir_all(config.user_dict_path.parent().unwrap()).await?;
    fs::create_dir_all(&config.file_dict_path).await?;

    let (service, socket) = LspService::new(|client| Backend::new(client, config));

    if args.stdio {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        Server::new(stdin, stdout, socket).serve(service).await;
    } else {
        let address = "127.0.0.1:4000";
        let listener = TcpListener::bind(address).await.unwrap();
        println!("Listening on {}", address);
        let (stream, _) = listener.accept().await.unwrap();
        let (read, write) = tokio::io::split(stream);
        Server::new(read, write, socket).serve(service).await;
    }

    Ok(())
}

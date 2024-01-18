mod generate_diagnostics;
mod server;
use clap::Parser;
use lsp_server::Connection;
use tracing::Level;

use crate::server::Server;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    stdio: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let (connection, io_threads) = if !args.stdio {
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish();

        tracing::subscriber::set_global_default(subscriber)?;

        Connection::listen("127.0.0.1:4000")?
    } else {
        Connection::stdio()
    };

    let mut server = Server::new(connection, io_threads)?;
    server.main_loop()?;
    server.join()?;

    Ok(())
}

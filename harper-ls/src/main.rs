mod server;

use lsp_server::Connection;
use tracing::Level;

use crate::server::Server;

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let (connection, io_threads) = Connection::listen("127.0.0.1:4000")?;
    let mut server = Server::new(connection, io_threads)?;
    server.main_loop()?;
    server.join()?;

    Ok(())
}

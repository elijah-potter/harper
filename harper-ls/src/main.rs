mod backend;
mod diagnostics;
use backend::Backend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::new(Backend::new);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout, socket).serve(service).await;
}

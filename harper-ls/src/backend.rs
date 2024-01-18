use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        notification::PublishDiagnostics, DidSaveTextDocumentParams, InitializeParams,
        InitializeResult, InitializedParams, MessageType, PublishDiagnosticsParams,
    },
    Client, LanguageServer,
};

use crate::diagnostics::generate_diagnostics;

pub struct Backend {
    client: Client,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult::default())
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let diagnostics = generate_diagnostics(&params.text_document.uri).unwrap();

        let result = PublishDiagnosticsParams {
            uri: params.text_document.uri,
            diagnostics,
            version: None,
        };

        self.client
            .send_notification::<PublishDiagnostics>(result)
            .await;
    }
}

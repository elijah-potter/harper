use std::ops::DerefMut;

use harper_core::{Dictionary, LintSet};
use tokio::{sync::Mutex, time::Instant};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        notification::PublishDiagnostics, CodeActionOrCommand, CodeActionParams,
        CodeActionProviderCapability, CodeActionResponse, DidChangeTextDocumentParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        InitializeParams, InitializeResult, InitializedParams, MessageType,
        PublishDiagnosticsParams, ServerCapabilities, Url,
    },
    Client, LanguageServer,
};

use crate::diagnostics::{generate_code_actions, generate_diagnostics};

pub struct Backend {
    client: Client,
    linter: Mutex<LintSet>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        let dictionary = Dictionary::new();
        let linter = Mutex::new(LintSet::new().with_standard(dictionary));

        Self { client, linter }
    }

    async fn publish_diagnostics(&self, url: &Url) {
        let start_time = Instant::now();
        let mut linter = self.linter.lock().await;

        let diagnostics = generate_diagnostics(url, linter.deref_mut()).unwrap();

        let result = PublishDiagnosticsParams {
            uri: url.clone(),
            diagnostics,
            version: None,
        };

        self.client
            .send_notification::<PublishDiagnostics>(result)
            .await;

        let end_time = Instant::now();

        let duration = end_time - start_time;

        self.client
            .log_message(
                MessageType::LOG,
                format!(
                    "Took {} ms to generate and publish diagnostics.",
                    duration.as_millis()
                ),
            )
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
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
        self.client
            .log_message(MessageType::INFO, "File saved!")
            .await;

        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "File opened!")
            .await;

        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "File changed!")
            .await;

        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_close(&self, _params: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "File closed!")
            .await;
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let mut linter = self.linter.lock().await;
        let actions =
            generate_code_actions(&params.text_document.uri, params.range, linter.deref_mut())?;

        self.client
            .log_message(MessageType::INFO, format!("{:?}", actions))
            .await;

        Ok(Some(
            actions
                .into_iter()
                .map(CodeActionOrCommand::CodeAction)
                .collect(),
        ))
    }
}

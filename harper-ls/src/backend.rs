use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use harper_core::parsers::Markdown;
use harper_core::{
    Dictionary,
    Document,
    FullDictionary,
    LintSet,
    Linter,
    MergedDictionary,
    Token,
    TokenKind
};
use serde_json::Value;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::{PublishDiagnostics, ShowMessage};
use tower_lsp::lsp_types::{
    CodeActionOrCommand,
    CodeActionParams,
    CodeActionProviderCapability,
    CodeActionResponse,
    Command,
    Diagnostic,
    DidChangeConfigurationParams,
    DidChangeTextDocumentParams,
    DidCloseTextDocumentParams,
    DidOpenTextDocumentParams,
    DidSaveTextDocumentParams,
    ExecuteCommandOptions,
    ExecuteCommandParams,
    InitializeParams,
    InitializeResult,
    InitializedParams,
    MessageType,
    PublishDiagnosticsParams,
    Range,
    ServerCapabilities,
    ShowMessageParams,
    TextDocumentSyncCapability,
    TextDocumentSyncKind,
    TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions,
    Url
};
use tower_lsp::{Client, LanguageServer};
use tracing::{error, info};

use crate::config::Config;
use crate::diagnostics::{lint_to_code_actions, lints_to_diagnostics};
use crate::dictionary_io::{load_dict, save_dict};
use crate::pos_conv::range_to_span;
use crate::tree_sitter_parser::TreeSitterParser;

#[derive(Default)]
struct DocumentState {
    document: Document,
    ident_dict: Arc<FullDictionary>,
    linter: LintSet
}

/// Deallocate
pub struct Backend {
    client: Client,
    static_dictionary: Arc<FullDictionary>,
    config: Mutex<Config>,
    doc_state: Mutex<HashMap<Url, DocumentState>>
}

impl Backend {
    pub fn new(client: Client, config: Config) -> Self {
        let dictionary = FullDictionary::create_from_curated();

        Self {
            client,
            static_dictionary: dictionary.into(),
            doc_state: Mutex::new(HashMap::new()),
            config: Mutex::new(config)
        }
    }

    /// Rewrites a path to a filename using the same conventions as
    /// [Neovim's undo-files](https://neovim.io/doc/user/options.html#'undodir').
    fn file_dict_name(url: &Url) -> PathBuf {
        let mut rewritten = String::new();

        // We assume all URLs are local files and have a base
        for seg in url.path_segments().unwrap() {
            rewritten.push_str(seg);
            rewritten.push('%');
        }

        rewritten.into()
    }

    async fn get_file_dict_path(&self, url: &Url) -> PathBuf {
        let config = self.config.lock().await;

        config.file_dict_path.join(Self::file_dict_name(url))
    }

    async fn load_file_dictionary(&self, url: &Url) -> FullDictionary {
        match load_dict(self.get_file_dict_path(url).await).await {
            Ok(dict) => dict,
            Err(err) => {
                error!("Problem loading file dictionary: {}", err);

                FullDictionary::new()
            }
        }
    }

    async fn save_file_dictionary(&self, url: &Url, dict: impl Dictionary) -> anyhow::Result<()> {
        Ok(save_dict(self.get_file_dict_path(url).await, dict).await?)
    }

    async fn load_user_dictionary(&self) -> FullDictionary {
        let config = self.config.lock().await;

        info!(
            "Loading user dictionary from `{}`",
            config
                .user_dict_path
                .clone()
                .into_os_string()
                .to_string_lossy()
        );

        match load_dict(&config.user_dict_path).await {
            Ok(dict) => dict,
            Err(err) => {
                error!("Problem loading user dictionary: {}", err);

                FullDictionary::new()
            }
        }
    }

    async fn save_user_dictionary(&self, dict: impl Dictionary) -> anyhow::Result<()> {
        let config = self.config.lock().await;

        info!(
            "Saving user dictionary to `{}`",
            config
                .user_dict_path
                .clone()
                .into_os_string()
                .to_string_lossy()
        );

        Ok(save_dict(&config.user_dict_path, dict).await?)
    }

    async fn generate_global_dictionary(&self) -> anyhow::Result<MergedDictionary<FullDictionary>> {
        let mut dict = MergedDictionary::new();
        dict.add_dictionary(self.static_dictionary.clone());
        let user_dict = self.load_user_dictionary().await;
        dict.add_dictionary(Arc::new(user_dict));
        Ok(dict)
    }

    async fn generate_file_dictionary(
        &self,
        url: &Url
    ) -> anyhow::Result<MergedDictionary<FullDictionary>> {
        let (global_dictionary, file_dictionary) = tokio::join!(
            self.generate_global_dictionary(),
            self.load_file_dictionary(url)
        );

        let mut global_dictionary = global_dictionary?;
        global_dictionary.add_dictionary(file_dictionary.into());

        Ok(global_dictionary)
    }

    async fn update_document_from_file(&self, url: &Url) -> anyhow::Result<()> {
        let content = match tokio::fs::read_to_string(url.path()).await {
            Ok(content) => content,
            Err(err) => {
                error!("Error updating document from file: {}", err);
                return Ok(());
            }
        };

        self.update_document(url, &content).await
    }

    async fn update_document(&self, url: &Url, text: &str) -> anyhow::Result<()> {
        let mut lock = self.doc_state.lock().await;

        // TODO: Only reset linter when underlying dictionaries change

        let mut doc_state = DocumentState {
            linter: LintSet::new().with_standard(self.generate_file_dictionary(url).await?),
            ..Default::default()
        };

        doc_state.document = if let Some(extension) = url.to_file_path().unwrap().extension() {
            if let Some(ts_parser) =
                TreeSitterParser::new_from_extension(&extension.to_string_lossy())
            {
                let source: Vec<char> = text.chars().collect();

                if let Some(new_dict) = ts_parser.create_ident_dict(source.as_slice()) {
                    let new_dict = Arc::new(new_dict);

                    if doc_state.ident_dict != new_dict {
                        doc_state.ident_dict = new_dict.clone();
                        let mut merged = self.generate_file_dictionary(url).await?;
                        merged.add_dictionary(new_dict);

                        doc_state.linter = LintSet::new().with_standard(merged);
                    }
                }

                Document::new_from_vec(source, Box::new(ts_parser))
            } else {
                Document::new(text, Box::new(Markdown))
            }
        } else {
            Document::new(text, Box::new(Markdown))
        };

        lock.insert(url.clone(), doc_state);

        Ok(())
    }

    async fn generate_code_actions(
        &self,
        url: &Url,
        range: Range
    ) -> Result<Vec<CodeActionOrCommand>> {
        let mut doc_states = self.doc_state.lock().await;
        let Some(doc_state) = doc_states.get_mut(url) else {
            return Ok(Vec::new());
        };

        let mut lints = doc_state.linter.lint(&doc_state.document);
        lints.sort_by_key(|l| l.priority);

        let source_chars = doc_state.document.get_full_content();

        // Find lints whose span overlaps with range
        let span = range_to_span(source_chars, range);

        let mut actions: Vec<CodeActionOrCommand> = lints
            .into_iter()
            .filter(|lint| lint.span.overlaps_with(span))
            .flat_map(|lint| lint_to_code_actions(&lint, url, source_chars))
            .collect();

        if let Some(Token {
            kind: TokenKind::Url,
            span,
            ..
        }) = doc_state.document.get_token_at_char_index(span.start)
        {
            actions.push(CodeActionOrCommand::Command(Command::new(
                "Open URL".to_string(),
                "HarperOpen".to_string(),
                Some(vec![doc_state.document.get_span_content_str(span).into()])
            )))
        }

        Ok(actions)
    }

    async fn generate_diagnostics(&self, url: &Url) -> Vec<Diagnostic> {
        let mut doc_states = self.doc_state.lock().await;
        let Some(doc_state) = doc_states.get_mut(url) else {
            return Vec::new();
        };

        let lints = doc_state.linter.lint(&doc_state.document);

        lints_to_diagnostics(doc_state.document.get_full_content(), &lints)
    }

    async fn publish_diagnostics(&self, url: &Url) {
        let client = self.client.clone();

        tokio::spawn(async move {
            client
                .send_notification::<ShowMessage>(ShowMessageParams {
                    typ: MessageType::INFO,
                    message: "Linting...".to_string()
                })
                .await
        });

        let diagnostics = self.generate_diagnostics(url).await;

        let result = PublishDiagnosticsParams {
            uri: url.clone(),
            diagnostics,
            version: None
        };

        self.client
            .send_notification::<PublishDiagnostics>(result)
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
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "HarperAddToUserDict".to_owned(),
                        "HarperAddToFileDict".to_owned(),
                        "HarperOpen".to_owned(),
                    ],
                    ..Default::default()
                }),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        will_save: None,
                        will_save_wait_until: None,
                        save: Some(TextDocumentSyncSaveOptions::Supported(true))
                    }
                )),

                ..Default::default()
            }
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

        let _ = self
            .update_document_from_file(&params.text_document.uri)
            .await;

        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "File opened!")
            .await;

        let _ = self
            .update_document_from_file(&params.text_document.uri)
            .await;

        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(last) = params.content_changes.last() else {
            return;
        };

        self.client
            .log_message(MessageType::INFO, "File changed!")
            .await;

        self.update_document(&params.text_document.uri, &last.text)
            .await
            .unwrap();
        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_close(&self, _params: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "File closed!")
            .await;
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        let mut string_args = params
            .arguments
            .into_iter()
            .map(|v| serde_json::from_value::<String>(v).unwrap());

        let Some(first) = string_args.next() else {
            return Ok(None);
        };

        info!("Received command: \"{}\"", params.command.as_str());

        match params.command.as_str() {
            "HarperAddToUserDict" => {
                let word = &first.chars().collect::<Vec<_>>();

                let Some(second) = string_args.next() else {
                    return Ok(None);
                };

                let file_url = second.parse().unwrap();

                let mut dict = self.load_user_dictionary().await;
                dict.append_word(word);
                if let Err(err) = self.save_user_dictionary(dict).await {
                    error!("Unable to save user dictionary: {}", err);
                }

                let _ = self.update_document_from_file(&file_url).await;
                self.publish_diagnostics(&file_url).await;
            }
            "HarperAddToFileDict" => {
                let word = &first.chars().collect::<Vec<_>>();

                let Some(second) = string_args.next() else {
                    return Ok(None);
                };

                let file_url = second.parse().unwrap();

                let mut dict = self.load_file_dictionary(&file_url).await;
                dict.append_word(word);

                if let Err(err) = self.save_file_dictionary(&file_url, dict).await {
                    error!("Unable to save file dictionary: {}", err);
                }

                let _ = self.update_document_from_file(&file_url).await;
                self.publish_diagnostics(&file_url).await;
            }
            "HarperOpen" => match open::that(&first) {
                Ok(()) => {
                    let message = format!(r#"Opened "{}""#, first);

                    self.client.log_message(MessageType::INFO, &message).await;

                    info!("{}", message);
                }
                Err(err) => {
                    self.client
                        .log_message(MessageType::ERROR, "Unable to open URL")
                        .await;
                    error!("Unable to open URL: {}", err);
                }
            },
            _ => ()
        }

        Ok(None)
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        info!("Changing user configuration.");

        let new_config = match Config::from_json_value(params.settings) {
            Ok(new_config) => new_config,
            Err(err) => {
                error!("Unable to change config: {}", err);
                return;
            }
        };

        let mut config = self.config.lock().await;
        *config = new_config;
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let actions = self
            .generate_code_actions(&params.text_document.uri, params.range)
            .await?;

        Ok(Some(actions))
    }
}

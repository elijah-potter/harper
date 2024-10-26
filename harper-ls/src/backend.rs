use std::collections::HashMap;
use std::path::{Component, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use harper_comments::CommentParser;
use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::{CollapseIdentifiers, IsolateEnglish, Markdown, Parser, PlainEnglish};
use harper_core::{
    Dictionary, Document, FullDictionary, MergedDictionary, Token, TokenKind, WordMetadata,
};
use harper_html::HtmlParser;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::PublishDiagnostics;
use tower_lsp::lsp_types::{
    CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability, CodeActionResponse,
    Command, ConfigurationItem, Diagnostic, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, ExecuteCommandOptions, ExecuteCommandParams, InitializeParams,
    InitializeResult, InitializedParams, MessageType, PublishDiagnosticsParams, Range,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions, Url,
};
use tower_lsp::{Client, LanguageServer};
use tracing::{error, info};

use crate::config::Config;
use crate::diagnostics::{lint_to_code_actions, lints_to_diagnostics};
use crate::dictionary_io::{load_dict, save_dict};
use crate::document_state::DocumentState;
use crate::git_commit_parser::GitCommitParser;
use crate::pos_conv::range_to_span;

pub struct Backend {
    client: Client,
    config: RwLock<Config>,
    doc_state: Mutex<HashMap<Url, DocumentState>>,
}

impl Backend {
    pub fn new(client: Client, config: Config) -> Self {
        Self {
            client,
            doc_state: Mutex::new(HashMap::new()),
            config: RwLock::new(config),
        }
    }

    /// Rewrites a path to a filename using the same conventions as
    /// [Neovim's undo-files](https://neovim.io/doc/user/options.html#'undodir').
    fn file_dict_name(url: &Url) -> Option<PathBuf> {
        let mut rewritten = String::new();

        // We assume all URLs are local files and have a base
        for seg in url.to_file_path().ok()?.components() {
            if !matches!(seg, Component::RootDir) {
                rewritten.push_str(&seg.as_os_str().to_string_lossy());
                rewritten.push('%');
            }
        }

        Some(rewritten.into())
    }

    /// Get the location of the file's specific dictionary
    async fn get_file_dict_path(&self, url: &Url) -> Option<PathBuf> {
        let config = self.config.read().await;

        Some(config.file_dict_path.join(Self::file_dict_name(url)?))
    }

    /// Load a speCific file's dictionary
    async fn load_file_dictionary(&self, url: &Url) -> Option<FullDictionary> {
        match load_dict(self.get_file_dict_path(url).await?).await {
            Ok(dict) => Some(dict),
            Err(_err) => Some(FullDictionary::new()),
        }
    }

    async fn save_file_dictionary(&self, url: &Url, dict: impl Dictionary) -> anyhow::Result<()> {
        Ok(save_dict(
            self.get_file_dict_path(url)
                .await
                .ok_or(anyhow!("Could not compute dictionary path."))?,
            dict,
        )
        .await?)
    }

    async fn load_user_dictionary(&self) -> FullDictionary {
        let config = self.config.read().await;

        match load_dict(&config.user_dict_path).await {
            Ok(dict) => dict,
            Err(_err) => FullDictionary::new(),
        }
    }

    async fn save_user_dictionary(&self, dict: impl Dictionary) -> anyhow::Result<()> {
        let config = self.config.read().await;

        Ok(save_dict(&config.user_dict_path, dict).await?)
    }

    async fn generate_global_dictionary(&self) -> anyhow::Result<MergedDictionary<FullDictionary>> {
        let mut dict = MergedDictionary::new();
        dict.add_dictionary(FullDictionary::curated());
        let user_dict = self.load_user_dictionary().await;
        dict.add_dictionary(Arc::new(user_dict));
        Ok(dict)
    }

    async fn generate_file_dictionary(
        &self,
        url: &Url,
    ) -> anyhow::Result<MergedDictionary<FullDictionary>> {
        let (global_dictionary, file_dictionary) = tokio::join!(
            self.generate_global_dictionary(),
            self.load_file_dictionary(url)
        );

        let Some(file_dictionary) = file_dictionary else {
            return Err(anyhow!("Unable to compute dictionary path."));
        };

        let mut global_dictionary = global_dictionary?;
        global_dictionary.add_dictionary(file_dictionary.into());

        Ok(global_dictionary)
    }

    async fn update_document_from_file(
        &self,
        url: &Url,
        language_id: Option<&str>,
    ) -> anyhow::Result<()> {
        let content = match tokio::fs::read_to_string(
            url.to_file_path()
                .map_err(|_| anyhow::format_err!("Could not extract file path."))?,
        )
        .await
        {
            Ok(content) => content,
            Err(err) => {
                error!("Error updating document from file: {}", err);
                return Ok(());
            }
        };

        self.update_document(url, &content, language_id).await
    }

    async fn update_document(
        &self,
        url: &Url,
        text: &str,
        language_id: Option<&str>,
    ) -> anyhow::Result<()> {
        self.pull_config().await;

        let mut doc_lock = self.doc_state.lock().await;
        let config_lock = self.config.read().await;

        let dict = Arc::new(self.generate_file_dictionary(url).await?);

        let doc_state = doc_lock.entry(url.clone()).or_insert(DocumentState {
            linter: LintGroup::new(config_lock.lint_config, dict.clone()),
            language_id: language_id.map(|v| v.to_string()),
            dict: dict.clone(),
            ..Default::default()
        });

        if doc_state.dict != dict {
            doc_state.dict = dict.clone();
            doc_state.linter = LintGroup::new(config_lock.lint_config, dict.clone());
        }

        let Some(language_id) = &doc_state.language_id else {
            doc_lock.remove(url);
            return Ok(());
        };

        let parser: Option<Box<dyn Parser>> =
            if let Some(ts_parser) = CommentParser::new_from_language_id(language_id) {
                let source: Vec<char> = text.chars().collect();
                let source = Arc::new(source);

                if let Some(new_dict) = ts_parser.create_ident_dict(source.as_slice()) {
                    let new_dict = Arc::new(new_dict);

                    if doc_state.ident_dict != new_dict {
                        doc_state.ident_dict = new_dict.clone();
                        let mut merged = self.generate_file_dictionary(url).await?;
                        merged.add_dictionary(new_dict);
                        let merged = Arc::new(merged);

                        doc_state.linter = LintGroup::new(config_lock.lint_config, merged.clone());
                        doc_state.dict = merged.clone();
                    }
                    Some(Box::new(CollapseIdentifiers::new(
                        Box::new(ts_parser),
                        &doc_state.dict,
                    )))
                } else {
                    Some(Box::new(ts_parser))
                }
            } else if language_id == "markdown" {
                Some(Box::new(Markdown))
            } else if language_id == "gitcommit" {
                Some(Box::new(GitCommitParser))
            } else if language_id == "html" {
                Some(Box::new(HtmlParser::default()))
            } else if language_id == "mail" {
                Some(Box::new(PlainEnglish))
            } else {
                None
            };

        match parser {
            None => {
                doc_lock.remove(url);
            }
            Some(mut parser) => {
                if self.config.read().await.isolate_english {
                    parser = Box::new(IsolateEnglish::new(parser, doc_state.dict.clone()));
                }

                doc_state.document = Document::new(text, &mut parser, &doc_state.dict);
            }
        }

        Ok(())
    }

    async fn generate_code_actions(
        &self,
        url: &Url,
        range: Range,
    ) -> Result<Vec<CodeActionOrCommand>> {
        let (config, mut doc_states) = tokio::join!(self.config.read(), self.doc_state.lock());
        let Some(doc_state) = doc_states.get_mut(url) else {
            return Ok(Vec::new());
        };

        let mut lints = doc_state.linter.lint(&doc_state.document);
        lints.sort_by_key(|l| l.priority);

        let source_chars = doc_state.document.get_full_content();

        // Find lints whole span overlaps with range
        let span = range_to_span(source_chars, range).with_len(1);

        let mut actions: Vec<CodeActionOrCommand> = lints
            .into_iter()
            .filter(|lint| lint.span.overlaps_with(span))
            .flat_map(|lint| {
                lint_to_code_actions(&lint, url, source_chars, &config.code_action_config)
            })
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
                Some(vec![doc_state.document.get_span_content_str(span).into()]),
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
        let config = self.config.read().await;

        lints_to_diagnostics(
            doc_state.document.get_full_content(),
            &lints,
            config.diagnostic_severity,
        )
    }

    async fn publish_diagnostics(&self, url: &Url) {
        let diagnostics = self.generate_diagnostics(url).await;

        let result = PublishDiagnosticsParams {
            uri: url.clone(),
            diagnostics,
            version: None,
        };

        self.client
            .send_notification::<PublishDiagnostics>(result)
            .await;
    }

    /// Update the configuration of the server and publish document updates that
    /// match it.
    async fn update_config_from_obj(&self, json_obj: Value) {
        let new_config = match Config::from_lsp_config(json_obj) {
            Ok(new_config) => new_config,
            Err(err) => {
                error!("Unable to change config: {}", err);
                return;
            }
        };

        {
            let mut config = self.config.write().await;
            *config = new_config;
        }
    }

    async fn pull_config(&self) {
        let mut new_config = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                section: None,
            }])
            .await
            .unwrap();

        if let Some(first) = new_config.pop() {
            self.update_config_from_obj(first).await;
        }
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
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                    },
                )),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Server initialized!")
            .await;

        self.pull_config().await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let _ = self
            .update_document_from_file(&params.text_document.uri, None)
            .await;

        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let _ = self
            .update_document(
                &params.text_document.uri,
                &params.text_document.text,
                Some(&params.text_document.language_id),
            )
            .await;

        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(last) = params.content_changes.last() else {
            return;
        };

        self.update_document(&params.text_document.uri, &last.text, None)
            .await
            .unwrap();
        self.publish_diagnostics(&params.text_document.uri).await;
    }

    async fn did_close(&self, _params: DidCloseTextDocumentParams) {}

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
                dict.append_word(word, WordMetadata::default());
                let _ = self.save_user_dictionary(dict).await;
                let _ = self.update_document_from_file(&file_url, None).await;
                self.publish_diagnostics(&file_url).await;
            }
            "HarperAddToFileDict" => {
                let word = &first.chars().collect::<Vec<_>>();

                let Some(second) = string_args.next() else {
                    return Ok(None);
                };

                let file_url = second.parse().unwrap();

                let Some(mut dict) = self.load_file_dictionary(&file_url).await else {
                    error!("Unable resolve dictionary path: {file_url}");
                    return Ok(None);
                };
                dict.append_word(word, WordMetadata::default());

                let _ = self.save_file_dictionary(&file_url, dict).await;
                let _ = self.update_document_from_file(&file_url, None).await;
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
            _ => (),
        }

        Ok(None)
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.update_config_from_obj(params.settings).await;
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let actions = self
            .generate_code_actions(&params.text_document.uri, params.range)
            .await?;

        Ok(Some(actions))
    }

    async fn shutdown(&self) -> Result<()> {
        let doc_states = self.doc_state.lock().await;

        // Clears the diagnostics for open buffers.
        for url in doc_states.keys() {
            let result = PublishDiagnosticsParams {
                uri: url.clone(),
                diagnostics: vec![],
                version: None,
            };

            self.client
                .send_notification::<PublishDiagnostics>(result)
                .await;
        }

        Ok(())
    }
}

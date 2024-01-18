use lsp_server::{
    Connection, ExtractError, IoThreads, Message, Notification, Request, RequestId, Response,
};
use lsp_types::{
    notification::{
        DidOpenTextDocument, DidSaveTextDocument, Notification as NotificationTrait,
        PublishDiagnostics,
    },
    request::{CodeActionRequest, GotoDefinition},
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionProviderCapability,
    CodeActionResponse, Diagnostic, DiagnosticOptions, GotoDefinitionResponse, InitializedParams,
    Location, OneOf, Position, PublishDiagnosticsParams, Range, ServerCapabilities, Url,
    WorkDoneProgressOptions,
};
use serde::Serialize;
use tracing::{error, info};

use crate::diagnostics::{generate_code_actions, generate_diagnostics};

pub struct Server {
    connection: Connection,
    io_threads: IoThreads,
    params: InitializedParams,
}

type RequestHandler = fn(server: &Server, req: &Request) -> anyhow::Result<()>;
type NotificationHandler = fn(server: &Server, notif: &Notification) -> anyhow::Result<()>;

impl Server {
    pub fn new(connection: Connection, io_threads: IoThreads) -> anyhow::Result<Self> {
        let server_capabilities = serde_json::to_value(ServerCapabilities {
            diagnostic_provider: Some(lsp_types::DiagnosticServerCapabilities::Options(
                DiagnosticOptions::default(),
            )),
            definition_provider: Some(OneOf::Left(true)),
            code_action_provider: Some(CodeActionProviderCapability::Options(
                lsp_types::CodeActionOptions {
                    code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    resolve_provider: None,
                },
            )),
            ..Default::default()
        })
        .unwrap();
        let initialization_params =
            serde_json::from_value(connection.initialize(server_capabilities)?)?;

        Ok(Self {
            connection,
            io_threads,
            params: initialization_params,
        })
    }

    pub fn main_loop(&mut self) -> anyhow::Result<()> {
        info!("Starting example main loop");
        for msg in &self.connection.receiver {
            info!("Got msg: {msg:?}");
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }

                    info!("Got request: {req:?}");

                    let handlers: [RequestHandler; 2] =
                        [Self::handle_goto, Self::handle_code_action];

                    for handler in handlers {
                        let res = handler(self, &req);

                        if let Err(err) = res {
                            error!("{}", err.to_string());
                        }
                    }
                }
                Message::Response(resp) => {
                    info!("Got response: {resp:?}");
                }
                Message::Notification(not) => {
                    info!("Got notification: {not:?}");

                    let handlers: [NotificationHandler; 2] = [Self::handle_open, Self::handle_save];

                    for handler in handlers {
                        let res = handler(self, &not);

                        if let Err(err) = res {
                            error!("{}", err.to_string());
                        }
                    }
                }
            };
        }

        Ok(())
    }

    pub fn join(self) -> anyhow::Result<()> {
        Ok(self.io_threads.join()?)
    }

    fn handle_save(&self, notif: &Notification) -> anyhow::Result<()> {
        let params = cast_notif::<DidSaveTextDocument>(notif.clone())?;

        self.publish_diagnostics(&params.text_document.uri)?;

        Ok(())
    }

    fn handle_open(&self, req: &Notification) -> anyhow::Result<()> {
        let params = cast_notif::<DidOpenTextDocument>(req.clone())?;

        self.publish_diagnostics(&params.text_document.uri)?;

        Ok(())
    }

    fn publish_diagnostics(&self, uri: &Url) -> anyhow::Result<()> {
        let diagnostics = generate_diagnostics(uri)?;

        let result = PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics,
            version: None,
        };

        let result = serde_json::to_value(result)?;
        self.connection
            .sender
            .send(Message::Notification(Notification {
                method: PublishDiagnostics::METHOD.to_string(),
                params: result,
            }))?;

        Ok(())
    }

    fn handle_goto(&self, req: &Request) -> anyhow::Result<()> {
        let (id, params) = cast_request::<GotoDefinition>(req.clone())?;

        info!("Got gotoDefinition request #{id}: {params:?}");
        let result = Some(GotoDefinitionResponse::Array(vec![Location {
            uri: params.text_document_position_params.text_document.uri,
            range: lsp_types::Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
        }]));

        self.send_response(result, id)?;

        Ok(())
    }

    fn handle_code_action(&self, req: &Request) -> anyhow::Result<()> {
        let (id, params) = cast_request::<CodeActionRequest>(req.clone())?;

        info!("Got code action request request #{id}: {params:?}");

        let actions = generate_code_actions(&params.text_document.uri, params.range)?;
        let response: CodeActionResponse = actions
            .into_iter()
            .map(CodeActionOrCommand::CodeAction)
            .collect();

        let result = Some(response);

        self.send_response(result, id)?;

        Ok(())
    }

    fn send_response<V: Serialize>(&self, result: V, id: RequestId) -> anyhow::Result<()> {
        let result = serde_json::to_value(result).unwrap();
        let resp = Response {
            id,
            result: Some(result),
            error: None,
        };

        self.connection.sender.send(Message::Response(resp))?;

        Ok(())
    }
}

fn cast_request<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn cast_notif<R>(notif: Notification) -> Result<R::Params, ExtractError<Notification>>
where
    R: lsp_types::notification::Notification,
    R::Params: serde::de::DeserializeOwned,
{
    notif.extract(R::METHOD)
}

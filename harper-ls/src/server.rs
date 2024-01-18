use anyhow::Ok;
use lsp_server::{
    Connection, ExtractError, IoThreads, Message, Notification, Request, RequestId, Response,
};
use lsp_types::{
    notification::{
        DidOpenTextDocument, DidSaveTextDocument, Notification as NotificationTrait,
        PublishDiagnostics,
    },
    request::GotoDefinition,
    Diagnostic, DiagnosticOptions, GotoDefinitionResponse, InitializedParams, Location, Position,
    PublishDiagnosticsParams, Range, ServerCapabilities,
};
use tracing::{error, info};

use crate::generate_diagnostics::generate_diagnostics;

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

                    let handlers: [RequestHandler; 1] = [Self::handle_goto];

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

    fn handle_open(&self, req: &Notification) -> anyhow::Result<()> {
        let params = cast_notif::<DidOpenTextDocument>(req.clone())?;

        dbg!(params);

        Ok(())
    }

    fn handle_save(&self, req: &Notification) -> anyhow::Result<()> {
        let params = cast_notif::<DidSaveTextDocument>(req.clone())?;

        let diagnostics = generate_diagnostics(params.text_document.uri.clone())?;

        let result = PublishDiagnosticsParams {
            uri: params.text_document.uri,
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
        let result = serde_json::to_value(&result).unwrap();
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

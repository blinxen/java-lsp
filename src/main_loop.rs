use std::error::Error;

use lsp_server::{Connection, Message};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
        Notification,
    },
    request::{GotoDefinition, Request},
};
use serde::de::DeserializeOwned;

use crate::{handlers, state::State};

pub fn start(
    connection: Connection,
    _params: serde_json::Value,
    state: &mut State,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    for msg in &connection.receiver {
        match msg {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    return Ok(());
                }

                match request.method.as_str() {
                    GotoDefinition::METHOD => {
                        handle_request(state, request, handlers::handle_go_to_definition)
                    }
                    _ => {}
                };
            }
            Message::Response(_reponse) => {}
            Message::Notification(notification) => {
                match notification.method.as_str() {
                    DidOpenTextDocument::METHOD => {
                        handle_notification::<DidOpenTextDocumentParams>(
                            state,
                            notification,
                            handlers::handle_did_open_text_document,
                        )
                    }
                    DidChangeTextDocument::METHOD => {
                        handle_notification::<DidChangeTextDocumentParams>(
                            state,
                            notification,
                            handlers::handle_did_change_text_document,
                        )
                    }
                    DidSaveTextDocument::METHOD => {
                        handle_notification::<DidSaveTextDocumentParams>(
                            state,
                            notification,
                            handlers::handle_did_save_text_document,
                        )
                    }
                    DidCloseTextDocument::METHOD => {
                        handle_notification::<DidCloseTextDocumentParams>(
                            state,
                            notification,
                            handlers::handle_did_close_text_document,
                        )
                    }
                    _ => {}
                };
            }
        }
    }
    Ok(())
}

fn handle_request<P>(
    state: &mut State,
    request: lsp_server::Request,
    handler: fn(&mut State, P) -> (),
) where
    P: DeserializeOwned,
{
    if let Ok(params) = serde_json::from_value::<P>(request.params) {
        handler(state, params);
    }
}

fn handle_notification<P>(
    state: &mut State,
    notification: lsp_server::Notification,
    handler: fn(&mut State, P) -> (),
) where
    P: DeserializeOwned,
{
    if let Ok(params) = serde_json::from_value::<P>(notification.params) {
        handler(state, params);
    }
}

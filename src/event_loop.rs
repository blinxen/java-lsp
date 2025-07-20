use std::error::Error;

use lsp_server::{Connection, Message};
use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    notification::{DidChangeTextDocument, DidOpenTextDocument, Notification},
};

use crate::state::State;

pub fn start(
    connection: Connection,
    _params: serde_json::Value,
    state: &mut State,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    for msg in &connection.receiver {
        eprintln!("got msg: {msg:?}");
        match msg {
            Message::Request(request) => {
                eprintln!("Got request {request:?}");
                // Don't do anything if server should be shutdown
                if connection.handle_shutdown(&request)? {
                    return Ok(());
                }
            }
            Message::Response(_response) => {
                eprintln!("Client responses are not handled at the moment");
            }
            Message::Notification(notification) => {
                match notification.method.as_str() {
                    DidOpenTextDocument::METHOD => {
                        // TODO: Allow excluding files with configuration
                        if let Ok(params) =
                            serde_json::from_value::<DidOpenTextDocumentParams>(notification.params)
                        {
                            state.register_document(
                                params.text_document.uri.as_str(),
                                &params.text_document.text,
                            );
                        }
                    }
                    DidChangeTextDocument::METHOD => {
                        if let Ok(params) = serde_json::from_value::<DidChangeTextDocumentParams>(
                            notification.params,
                        ) {
                            for change in params.content_changes {
                                state.update_document(
                                    params.text_document.uri.as_str(),
                                    params.text_document.version,
                                    change.range.as_ref(),
                                    &change.text,
                                );
                            }
                        }
                    }
                    _ => {}
                };
            }
        }
    }
    Ok(())
}

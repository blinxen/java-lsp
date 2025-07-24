use std::error::Error;

use lsp_server::{Connection, Message};
use lsp_types::{
    Diagnostic, DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    Position, PublishDiagnosticsParams, Range,
    notification::{
        DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument, Notification,
        PublishDiagnostics,
    },
};

use crate::{document::Document, state::State};

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
                                params.text_document.uri.clone(),
                                &params.text_document.text,
                            );
                            compile_and_publish_diagnostics(
                                &connection,
                                state
                                    .get_document(params.text_document.uri.as_str())
                                    .expect(
                                        "Registered document could not be found for some reason",
                                    ),
                            );
                        }
                    }
                    DidChangeTextDocument::METHOD => {
                        if let Ok(params) = serde_json::from_value::<DidChangeTextDocumentParams>(
                            notification.params,
                        ) {
                            for change in params.content_changes {
                                state.update_document(
                                    params.text_document.uri.clone(),
                                    params.text_document.version,
                                    change.range.as_ref(),
                                    &change.text,
                                );
                            }
                        }
                    }
                    DidSaveTextDocument::METHOD => {
                        if let Ok(params) =
                            serde_json::from_value::<DidSaveTextDocumentParams>(notification.params)
                        {
                            if let Some(document) =
                                state.get_document(params.text_document.uri.as_str())
                            {
                                compile_and_publish_diagnostics(&connection, document);
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

fn compile_and_publish_diagnostics(connection: &Connection, document: &mut Document) {
    document.compile();
    let errors = document
        .compile_errors()
        .iter()
        .map(|error| {
            let position = Position::new(error.row - 1, error.column);
            Diagnostic::new_simple(Range::new(position, position), error.error_message.clone())
        })
        .collect::<Vec<Diagnostic>>();

    if let Ok(params) = serde_json::to_value(PublishDiagnosticsParams::new(
        document.uri().clone(),
        errors,
        Some(document.version()),
    )) {
        eprintln!("{:?}", params.clone());
        // TODO: This can fail and try_send will return the method that
        // could not be sent. We could probably try to send it again
        // later
        let _ = connection
            .sender
            .try_send(Message::Notification(lsp_server::Notification {
                method: PublishDiagnostics::METHOD.to_string(),
                params,
            }));
    }
}

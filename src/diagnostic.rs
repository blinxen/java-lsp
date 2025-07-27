use std::collections::HashMap;

use crossbeam_channel::Sender;
use lsp_server::Message;
use lsp_types::{
    Diagnostic, Position, PublishDiagnosticsParams, Range, Url,
    notification::{Notification, PublishDiagnostics},
};

use crate::compiler::CompileError;

pub fn publish_compile_errors(
    sender: &Sender<Message>,
    compile_errors: HashMap<Url, Vec<CompileError>>,
) {
    for (uri, errors) in compile_errors {
        let errors = errors
            .iter()
            .map(|error| {
                let position = Position::new(error.row.saturating_sub(1), error.column);
                Diagnostic::new_simple(Range::new(position, position), error.error_message.clone())
            })
            .collect::<Vec<Diagnostic>>();

        if let Ok(params) = serde_json::to_value(PublishDiagnosticsParams::new(
            uri, errors, // TODO: Do we care about this?
            None,
        )) {
            // TODO: This can fail and try_send will return the method that
            // could not be sent. We could probably try to send it again
            // later
            let _ = sender.try_send(Message::Notification(lsp_server::Notification {
                method: PublishDiagnostics::METHOD.to_string(),
                params,
            }));
        }
    }
}

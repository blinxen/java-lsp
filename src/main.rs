mod classfile;
mod classpath_indexer;
mod compiler;
mod configuration;
mod diagnostic;
mod document;
mod errors;
mod gradle;
mod handlers;
mod main_loop;
mod maven;
mod state;
mod tree_sitter;

use compiler::Compiler;
use lsp_server::Connection;
use lsp_types::{ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind};
use state::State;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("Starting java lsp server");
    configuration::initialize_data_directory();

    let (connection, io_threads) = Connection::stdio();
    let mut state = State::new(connection.sender.clone(), Compiler::new());

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        definition_provider: Some(lsp_types::OneOf::Left(true)),
        ..Default::default()
    })
    .unwrap();

    let initialization_params = match connection.initialize(server_capabilities) {
        Ok(it) => it,
        Err(e) => {
            if e.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(e.into());
        }
    };

    main_loop::start(connection, initialization_params, &mut state)?;
    io_threads.join()?;

    eprintln!("Shutting down server");
    Ok(())
}

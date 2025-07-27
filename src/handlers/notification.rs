use std::collections::HashMap;

use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, Url,
};

use crate::{compiler::CompileError, diagnostic, state::State};

pub fn handle_did_open_text_document(state: &mut State, params: DidOpenTextDocumentParams) {
    // TODO: Allow excluding files with configuration
    state.register_document(params.text_document.uri.clone(), &params.text_document.text);
    compile_and_publish_compile_errors(state);
}

pub fn handle_did_change_text_document(state: &mut State, params: DidChangeTextDocumentParams) {
    for change in params.content_changes {
        state.update_document(
            params.text_document.uri.clone(),
            params.text_document.version,
            change.range.as_ref(),
            &change.text,
        );
    }
}

pub fn handle_did_save_text_document(state: &mut State, params: DidSaveTextDocumentParams) {
    if state
        .get_document(params.text_document.uri.as_str())
        .is_some()
    {
        compile_and_publish_compile_errors(state);
    }
}

pub fn handle_did_close_text_document(state: &mut State, params: DidCloseTextDocumentParams) {
    state.unregister_document(params.text_document.uri);
}

fn compile_and_publish_compile_errors(state: &mut State) {
    let errors = state.compiler.compile(false);
    let fixed_documents: HashMap<Url, Vec<CompileError>> = state
        .registered_documents()
        .into_iter()
        .filter(|path| !errors.contains_key(path))
        .map(|url| (url.clone(), Vec::new()))
        .collect();

    diagnostic::publish_compile_errors(&state.sender, errors);
    // Clear documents that were fixed
    diagnostic::publish_compile_errors(&state.sender, fixed_documents);
}

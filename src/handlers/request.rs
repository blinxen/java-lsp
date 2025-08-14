use lsp_types::GotoDefinitionParams;

use crate::state::State;

pub fn handle_go_to_definition(state: &mut State, params: GotoDefinitionParams) {
    if let Some(document) = state.document(
        params
            .text_document_position_params
            .text_document
            .uri
            .as_str(),
    ) {
        document.symbol_at_position(params.text_document_position_params.position);
    }
}

use std::collections::HashMap;

use lsp_types::Range;

use crate::document::Document;

#[derive(Default)]
pub struct State {
    documents: HashMap<String, Document>,
}

impl State {
    pub fn register_document(&mut self, uri: &str, content: &str) {
        self.documents
            .insert(uri.to_string(), Document::new(0, content));
    }

    pub fn update_document(&mut self, uri: &str, version: i32, range: Option<&Range>, text: &str) {
        if let Some(document) = self.documents.get_mut(uri) {
            // Ignore change requests that are older than the internal state
            if !document.should_update(version) {
                return;
            }

            if let Some(range) = range {
                // Update the document
                let start_index = document.position_index(range.start);
                let end_index = document.position_index(range.end);

                if end_index < start_index {
                    eprintln!(
                        "ERROR: Could not update document because end {end_index} is less than {start_index}"
                    );
                } else {
                    document.update(start_index, end_index, text);
                }
            } else {
                // If range is none then just replace the whole document
                self.documents
                    .insert(uri.to_string(), Document::new(version, text));
            }
        }
    }
}

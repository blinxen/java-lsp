use std::collections::HashMap;

use crate::{compiler::Compiler, document::Document};
use crossbeam_channel::Sender;
use lsp_server::Message;
use lsp_types::{Range, Url};

pub struct State {
    documents: HashMap<String, Document>,
    pub compiler: Compiler,
    pub sender: Sender<Message>,
}

impl State {
    pub fn new(sender: Sender<Message>, compiler: Compiler) -> Self {
        State {
            sender,
            documents: HashMap::new(),
            compiler,
        }
    }

    pub fn registered_documents(&mut self) -> Vec<&Url> {
        self.documents
            .values()
            .map(|document| document.uri())
            .collect()
    }

    pub fn get_document(&mut self, uri: &str) -> Option<&mut Document> {
        self.documents.get_mut(uri)
    }

    pub fn register_document(&mut self, uri: Url, content: &str) {
        self.documents
            .insert(uri.to_string(), Document::new(uri, 0, content));
    }

    pub fn update_document(&mut self, uri: Url, version: i32, range: Option<&Range>, text: &str) {
        if let Some(document) = self.documents.get_mut(uri.as_str()) {
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
                    .insert(uri.to_string(), Document::new(uri, version, text));
            }
        }
    }

    pub fn unregister_document(&mut self, uri: Url) {
        self.documents.remove(&uri.to_string());
    }
}

use crate::{
    classfile::Classfile, classpath_indexer, compiler::Compiler, document::Document,
    errors::DocumentError,
};
use crossbeam_channel::Sender;
use lsp_server::Message;
use lsp_types::{Range, Url};
use std::collections::HashMap;

pub struct State {
    documents: HashMap<String, Document>,
    classes: HashMap<String, Classfile>,
    pub compiler: Compiler,
    pub sender: Sender<Message>,
}

impl State {
    pub fn new(sender: Sender<Message>, compiler: Compiler) -> Self {
        State {
            documents: HashMap::new(),
            classes: classpath_indexer::index(compiler.classpath()),
            compiler,
            sender,
        }
    }

    pub fn documents_uri(&mut self) -> Vec<&Url> {
        self.documents
            .values()
            .map(|document| &document.uri)
            .collect()
    }

    pub fn document(&mut self, uri: &str) -> Option<&mut Document> {
        self.documents.get_mut(uri)
    }

    pub fn register_document(&mut self, uri: Url, content: &str) -> Result<(), DocumentError> {
        self.documents
            .insert(uri.to_string(), Document::new(uri, content)?);

        Ok(())
    }

    pub fn update_document(
        &mut self,
        uri: Url,
        version: i32,
        range: Option<&Range>,
        text: &str,
    ) -> Result<(), DocumentError> {
        // Ignore change requests that are older than the internal state
        if let Some(document) = self.documents.get_mut(uri.as_str())
            && document.should_update(version)
        {
            if let Some(range) = range {
                document.update(range.start, range.end, text)?;
            } else {
                // If range is none then just replace the whole document
                self.documents
                    .insert(uri.to_string(), Document::new(uri, text)?);
            }
        }

        Ok(())
    }

    pub fn unregister_document(&mut self, uri: Url) {
        self.documents.remove(&uri.to_string());
    }
}

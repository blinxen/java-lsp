use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("Could not initialize tree sitter parser")]
    TreeSitterParserError(#[from] tree_sitter::LanguageError),
    #[error("Could not parse source code for {0}")]
    TreeSitterParseError(String),
}

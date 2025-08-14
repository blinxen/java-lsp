use std::ops::Range;

use lsp_types::{Position, Url};
use ropey::Rope;
use tree_sitter::{InputEdit, Parser, Point, Tree};

use crate::errors::DocumentError;
use crate::tree_sitter::{collect_imports, find_node_by_point};

pub struct Document {
    version: i32,
    content: Rope,
    tree: Tree,
    parser: Parser,
    pub uri: Url,
}

impl Document {
    pub fn new(uri: Url, content: &str) -> Result<Self, DocumentError> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_java::LANGUAGE.into())?;
        let tree = parser
            .parse(content, None)
            .ok_or(DocumentError::TreeSitterParseError(uri.to_string()))?;

        Ok(Document {
            uri,
            parser,
            tree,
            version: 0,
            content: Rope::from_str(content),
        })
    }

    pub fn update(
        &mut self,
        start: Position,
        end: Position,
        updated_content: &str,
    ) -> Result<(), DocumentError> {
        let start_index = self.position_index(start);
        let end_index = self.position_index(end);

        if start_index < end_index {
            self.content.remove(start_index..end_index);
            if !updated_content.is_empty() {
                self.content.insert(start_index, updated_content);
            }
            self.tree.edit(&InputEdit {
                start_byte: start_index,
                old_end_byte: end_index,
                new_end_byte: start_index + updated_content.len(),
                start_position: Point::new(0, 0),
                old_end_position: Point::new(0, 0),
                new_end_position: Point::new(0, 0),
            });
        } else {
            self.content.remove(end_index..start_index);
            if !updated_content.is_empty() {
                self.content.insert(end_index, updated_content);
            }
            self.tree.edit(&InputEdit {
                start_byte: end_index,
                old_end_byte: start_index,
                new_end_byte: end_index + updated_content.len(),
                start_position: Point::new(0, 0),
                old_end_position: Point::new(0, 0),
                new_end_position: Point::new(0, 0),
            });
        }

        self.tree = self
            .parser
            // TODO: This seems to be expensive, can it be done better?
            .parse(self.content.bytes().collect::<Vec<u8>>(), Some(&self.tree))
            .ok_or(DocumentError::TreeSitterParseError(self.uri.to_string()))?;

        Ok(())
    }

    pub fn should_update(&self, version: i32) -> bool {
        self.version < version
    }

    pub fn symbol_at_position(&self, position: Position) -> Option<&str> {
        let imports = collect_imports(self.tree.walk())
            .iter()
            .map(|node| self.slice_by_range(node.byte_range()))
            .collect::<Vec<&str>>();
        let node = find_node_by_point(
            self.tree.walk(),
            Point::new(position.line as usize, position.character as usize),
        );

        match node.kind() {
            "type_identifier" => {
                imports.iter().find(|item| {
                    item.ends_with(&format!(".{}", self.slice_by_range(node.byte_range())))
                }).map(|element| *element)
            }
            "identifier" => {
                None
            },
            _ => None
        }
    }

    fn slice_by_range(&self, range: Range<usize>) -> &str {
        self.content.byte_slice(range).as_str().unwrap()
    }

    /// Get document index from [`Position`]
    fn position_index(&self, position: Position) -> usize {
        // TODO: This can panic and should be handled better but I would like to see when
        // this actually happens.
        self.content.line_to_char(position.line as usize) + position.character as usize
    }
}

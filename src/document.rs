use lsp_types::Position;
use ropey::Rope;

pub struct Document {
    pub version: i32,
    content: Rope,
}

impl Document {
    pub fn new(version: i32, content: &str) -> Self {
        Document {
            version,
            content: Rope::from_str(content),
        }
    }

    pub fn update(&mut self, start: usize, end: usize, updated_content: &str) {
        self.content.remove(start..end);

        if !updated_content.is_empty() {
            if start < self.content.len_chars() {
                self.content.insert(start, updated_content);
            } else {
                self.content
                    .insert(self.content.len_chars(), updated_content);
            }
        }
    }

    pub fn should_update(&self, version: i32) -> bool {
        self.version < version
    }

    /// Get document index from [`Position`]
    pub fn position_index(&self, position: Position) -> usize {
        // TODO: This can panic and should be handled better but I would like to see when
        // this actually happens.
        self.content.line_to_char(position.line as usize) + position.character as usize
    }
}

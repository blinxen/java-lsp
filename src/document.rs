use crate::{gradle, maven};
use lsp_types::{Position, Url};
use ropey::Rope;
use std::{path::Path, process::Command};

#[derive(Debug)]
pub struct CompileError {
    pub row: u32,
    pub column: u32,
    pub error_message: String,
}

pub struct Document {
    uri: Url,
    version: i32,
    content: Rope,
    compile_errors: Vec<CompileError>,
}

impl Document {
    pub fn new(uri: Url, version: i32, content: &str) -> Self {
        Document {
            uri,
            version,
            content: Rope::from_str(content),
            compile_errors: Vec::new(),
        }
    }

    pub fn compile_errors(&self) -> &Vec<CompileError> {
        &self.compile_errors
    }

    pub fn uri(&self) -> &Url {
        &self.uri
    }

    pub fn version(&self) -> i32 {
        self.version
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

    pub fn compile(&mut self) {
        if let Ok(path) = self.uri.to_file_path()
            && path.exists()
        {
            // TODO: Support annotation processing
            let output = Command::new("javac")
                .arg("--class-path")
                .arg(Self::determine_classpath())
                .arg("-d")
                .arg("target/classes")
                // .arg("-Xlint:all")
                // .arg("-Xdoclint:all")
                .arg("-Xdiags:verbose")
                .arg(&path)
                .output();

            if let Ok(output) = output
                && let Ok(stderr) = str::from_utf8(&output.stderr)
            {
                self.compile_errors.clear();
                let mut lines = stderr.lines();

                while let Some(line) = lines.next() {
                    if line.starts_with(path.to_str().unwrap_or_default())
                        && let Some(column_line) = lines.next()
                    {
                        let mut parts = line.split(":");
                        self.compile_errors.push(CompileError {
                            row: parts.nth(1).unwrap_or("0").parse::<u32>().unwrap_or(0),
                            column: column_line
                                .split_once("^")
                                .unwrap_or_default()
                                .0
                                .chars()
                                .take_while(|&c| c.is_whitespace())
                                .count() as u32,
                            error_message: parts.last().unwrap_or("").to_string(),
                        });
                    }
                }
            }
        }
    }

    fn determine_classpath() -> String {
        let mut classpath = String::new();

        if Path::new("./pom.xml").exists() {
            classpath = maven::generate_claspath();
        } else if Path::new("./build.gradle").exists()
            || Path::new("./build.gradle.kt").exists()
            || Path::new("./build.gradle.kts").exists()
        {
            classpath = gradle::generate_claspath();
        }

        classpath
    }
}

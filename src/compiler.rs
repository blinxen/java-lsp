use std::{collections::HashMap, path::Path, process::Command};

use lsp_types::Url;
use walkdir::{DirEntry, WalkDir};

use crate::{gradle, maven};

enum ProjectKind {
    Maven,
    Gradle,
    Javac,
}

#[derive(Debug)]
pub struct CompileError {
    pub row: u32,
    pub column: u32,
    pub error_message: String,
}

pub struct Compiler {
    project_kind: ProjectKind,
    classpath: String,
}

impl Compiler {
    pub fn new() -> Self {
        let project_kind = determine_project_kind();
        let classpath = determine_classpath(&project_kind);

        Compiler {
            project_kind,
            classpath,
        }
    }

    pub fn compile(&self, force_all: bool) -> HashMap<Url, Vec<CompileError>> {
        let mut errors: HashMap<Url, Vec<CompileError>> = HashMap::new();
        let class_file_directory = self.class_file_target_directory();

        // TODO: Support annotation processing
        let output = Command::new("javac")
            .arg("--class-path")
            .arg(&self.classpath)
            .arg("-d")
            .arg(&class_file_directory)
            // .arg("-Xlint:all")
            // .arg("-Xdoclint:all")
            .arg("-Xdiags:verbose")
            .args(find_files_to_compile(class_file_directory, force_all))
            .output();

        if let Ok(output) = output
            && let Ok(stderr) = str::from_utf8(&output.stderr)
        {
            let mut lines = stderr.lines();
            while let Some(line) = lines.next() {
                if !line
                    .chars()
                    .next()
                    .map(|c| c.is_whitespace())
                    .unwrap_or(false)
                    && line.contains(".java:")
                {
                    let mut parts = line.split(":");
                    lines.next();

                    if let Some(path) = parts.nth(0)
                        && let Ok(url) = Url::parse(&(String::from("file://") + path))
                        && let Some(column_line) = lines.next()
                    {
                        let compilation_error = CompileError {
                            row: parts.nth(0).unwrap_or("0").parse::<u32>().unwrap_or(0),
                            column: column_line
                                .split_once("^")
                                .unwrap_or_default()
                                .0
                                .chars()
                                .take_while(|&c| c.is_whitespace())
                                .count() as u32,
                            error_message: parts.last().unwrap_or("").to_string(),
                        };

                        if let Some(error) = errors.get_mut(&url) {
                            error.push(compilation_error);
                        } else {
                            errors.insert(url, vec![compilation_error]);
                        }
                    }
                }
            }
        } else {
            eprintln!("Compilation was unsuccessfull");
        }

        errors
    }

    fn class_file_target_directory(&self) -> String {
        match self.project_kind {
            ProjectKind::Maven | ProjectKind::Javac => String::from("target/classes"),
            ProjectKind::Gradle => String::from("build/classes"),
        }
    }

    pub fn refresh_classpath(&mut self) {
        self.classpath = determine_classpath(&self.project_kind);
    }
}

fn determine_project_kind() -> ProjectKind {
    if Path::new("./pom.xml").exists() {
        ProjectKind::Maven
    } else if Path::new("./build.gradle").exists()
        || Path::new("./build.gradle.kt").exists()
        || Path::new("./build.gradle.kts").exists()
    {
        ProjectKind::Gradle
    } else {
        ProjectKind::Javac
    }
}

fn determine_classpath(kind: &ProjectKind) -> String {
    match kind {
        ProjectKind::Maven => maven::generate_claspath(),
        ProjectKind::Gradle => gradle::generate_claspath(),
        ProjectKind::Javac => String::new(),
    }
}

fn find_files_to_compile(class_file_directory: String, force_all: bool) -> Vec<String> {
    let class_files = WalkDir::new(class_file_directory)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().is_file()
                && entry
                    .path()
                    .extension()
                    .map(|ext| ext == "class")
                    .unwrap_or(false)
        })
        .map(|entry| {
            (
                std::fs::canonicalize(entry.path())
                    .unwrap()
                    .display()
                    .to_string()
                    .replace(".class", ""),
                entry,
            )
        })
        .collect::<HashMap<String, DirEntry>>();

    // TODO: Should we handle this? If yes then it should probably be done in the earlier stages
    WalkDir::new(std::env::current_dir().unwrap_or_default())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().is_file()
                && entry
                    .path()
                    .extension()
                    .map(|ext| ext == "java")
                    .unwrap_or(false)
                && (force_all || should_build_file(&class_files, entry) == Some(true))
        })
        .map(|entry| entry.path().display().to_string())
        .collect()
}

fn should_build_file(
    class_files: &HashMap<String, DirEntry>,
    java_file: &DirEntry,
) -> Option<bool> {
    let mut should_build = true;

    if let Some(file) =
        class_files.get(&java_file.path().display().to_string().replace(".java", ""))
        && let Ok(class_file_metadata) = file.metadata()
    {
        let java_file_metadata = java_file.metadata().ok()?;

        if let Ok(class_file_modified) = class_file_metadata.modified()
            && let Ok(java_file_modified) = java_file_metadata.modified()
        {
            if class_file_modified >= java_file_modified {
                should_build = false;
            }
        } else if let Ok(class_file_created) = class_file_metadata.created()
            && let Ok(java_file_created) = java_file_metadata.created()
            && class_file_created >= java_file_created
        {
            should_build = false;
        }
    }

    Some(should_build)
}

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub const GRADLE_CLASSPATH_TASK_NAME: &str = "generateClasspath";
pub const GRADLE_INIT_SCRIPT_FILE_NAME: &str = "gradle-init-script.gradle";
pub const GRADLE_INIT_SCRIPT: &str = r#"
gradle.projectsEvaluated {
    allprojects {
        if (plugins.hasPlugin('java')) {
            tasks.register("generateClasspath") {
                doLast {
                    def mainClasspath = configurations.findByName('compileClasspath') ?: files()
                    def testClasspath = configurations.findByName('testCompileClasspath') ?: files()
                    def classpath = (mainClasspath.files + testClasspath.files).toSet()

                    println classpath.collect { it.absolutePath }.join(File.pathSeparator)
                }
            }
        }
    }
}
"#;

pub fn data_directory() -> PathBuf {
    let base_path = PathBuf::from(env::var("HOME").unwrap_or_default()).join(".cache");

    if base_path.exists() {
        base_path.join("java-lsp")
    } else {
        PathBuf::from("/tmp/java-lsp")
    }
}

pub fn initialize_data_directory() {
    let data_dir = data_directory();
    // TODO: Remove all usages of .expect
    std::fs::create_dir_all(&data_dir).expect("Could not initialize / find data directory");

    let mut gradle_init_script = File::create(data_dir.join(GRADLE_INIT_SCRIPT_FILE_NAME))
        .expect("Could not create init script for gradle");
    gradle_init_script
        .write_all(GRADLE_INIT_SCRIPT.as_bytes())
        .expect("");
}

pub fn gradle_init_script_path() -> Option<PathBuf> {
    if data_directory().join(GRADLE_INIT_SCRIPT_FILE_NAME).exists() {
        Some(data_directory().join(GRADLE_INIT_SCRIPT_FILE_NAME))
    } else {
        None
    }
}

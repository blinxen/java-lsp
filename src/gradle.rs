use crate::configuration::{GRADLE_CLASSPATH_TASK_NAME, gradle_init_script_path};
use std::process::Command;

pub fn generate_claspath() -> String {
    let mut classpath = String::new();

    // TODO: try to search for gradle and gradlew
    if let Ok(output) = Command::new("gradle")
        .arg(GRADLE_CLASSPATH_TASK_NAME)
        .arg("--quiet")
        .arg("--init-script")
        .arg(gradle_init_script_path().unwrap_or_default())
        .output()
    {
        classpath = String::from_utf8(output.stdout).unwrap_or_default();
    }

    classpath
}

use std::process::Command;

pub fn generate_claspath() -> String {
    let mut classpath = String::from("target/classes");

    // TODO: try to search for mvn or mvnw
    if let Ok(output) = Command::new("mvn")
        .arg("--quiet")
        .arg("dependency:build-classpath")
        .arg("-Dmdep.outputFile=/dev/stdout")
        .output()
    {
        classpath += ":";
        classpath += str::from_utf8(&output.stdout).unwrap_or_default();
    }

    classpath
}

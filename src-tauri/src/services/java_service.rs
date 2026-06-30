use std::{env, io::ErrorKind, path::PathBuf, process::Command};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct JavaDetection {
    pub found: bool,
    pub path: String,
    pub version: String,
}

pub fn detect_java() -> Result<JavaDetection, String> {
    let mut candidates = Vec::new();
    if let Ok(java_home) = env::var("JAVA_HOME") {
        let executable = if cfg!(windows) { "java.exe" } else { "java" };
        candidates.push(PathBuf::from(java_home).join("bin").join(executable));
    }
    candidates.push(PathBuf::from(if cfg!(windows) {
        "java.exe"
    } else {
        "java"
    }));

    let mut last_error = String::new();
    for candidate in candidates {
        match probe_java(candidate) {
            Ok(Some(detection)) => return Ok(detection),
            Ok(None) => {}
            Err(error) => last_error = error,
        }
    }

    Ok(JavaDetection {
        found: false,
        path: String::new(),
        version: if last_error.is_empty() {
            "Java was not found in JAVA_HOME or PATH".to_string()
        } else {
            last_error
        },
    })
}

fn probe_java(path: PathBuf) -> Result<Option<JavaDetection>, String> {
    let output = match Command::new(&path).arg("-version").output() {
        Ok(output) => output,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };

    if !output.status.success() {
        return Ok(None);
    }

    let text = String::from_utf8_lossy(&output.stderr);
    let version = text.lines().next().unwrap_or("Java detected").to_string();
    Ok(Some(JavaDetection {
        found: true,
        path: path.to_string_lossy().to_string(),
        version,
    }))
}

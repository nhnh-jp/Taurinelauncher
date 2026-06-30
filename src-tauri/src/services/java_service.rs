use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct JavaDetection {
    pub found: bool,
    pub path: String,
    pub version: String,
}

pub fn detect_java() -> Result<JavaDetection, String> {
    Ok(JavaDetection {
        found: false,
        path: "auto".to_string(),
        version: "Phase 4で実装".to_string(),
    })
}

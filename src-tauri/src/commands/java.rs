use crate::services::java_service::{self, JavaDetection};

#[tauri::command]
pub fn detect_java() -> Result<JavaDetection, String> { java_service::detect_java() }

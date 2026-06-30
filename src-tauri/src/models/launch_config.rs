use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LaunchResult {
    pub command_preview: String,
    pub log_path: String,
}

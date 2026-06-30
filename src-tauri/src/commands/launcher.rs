use crate::{models::launch_config::LaunchResult, services::launch_service};

#[tauri::command]
pub fn launch_minecraft(profile_path: String) -> Result<LaunchResult, String> { launch_service::launch_minecraft(profile_path) }

#[tauri::command]
pub fn stream_logs(profile_path: String) -> Result<Vec<String>, String> { launch_service::stream_logs(profile_path) }

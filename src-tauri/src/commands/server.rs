use crate::{models::server_config::ServerConfig, services::server_service};

#[tauri::command]
pub fn list_servers() -> Result<Vec<String>, String> { server_service::phase_three() }

#[tauri::command]
pub fn read_server_config(_server_name: String) -> Result<ServerConfig, String> { server_service::phase_three() }

#[tauri::command]
pub fn create_server_profile(_server_name: String) -> Result<String, String> { server_service::phase_three() }

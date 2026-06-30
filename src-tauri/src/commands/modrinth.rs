use serde::Serialize;

use crate::{models::mod_info::ModInfo, services::modrinth_service};

#[derive(Debug, Clone, Serialize)]
pub struct ModrinthSearchResult {
    pub project_id: String,
    pub title: String,
    pub description: String,
}

#[tauri::command]
pub fn list_installed_mods(profile_path: String) -> Result<Vec<ModInfo>, String> {
    modrinth_service::list_installed_mods(profile_path)
}

#[tauri::command]
pub fn search_modrinth(
    _query: String,
    _version: String,
    _loader: String,
) -> Result<Vec<ModrinthSearchResult>, String> {
    Err("Modrinth search is not implemented yet".to_string())
}

#[tauri::command]
pub fn get_modrinth_versions(
    _project_id: String,
    _version: String,
    _loader: String,
) -> Result<Vec<String>, String> {
    Err("Modrinth version lookup is not implemented yet".to_string())
}

#[tauri::command]
pub fn install_mod(
    _profile_path: String,
    _project_id: String,
    _version_id: String,
) -> Result<(), String> {
    Err("Modrinth download is not implemented yet".to_string())
}

#[tauri::command]
pub fn remove_mod(profile_path: String, file_name: String) -> Result<(), String> {
    modrinth_service::remove_mod(profile_path, file_name)
}

#[tauri::command]
pub fn enable_mod(profile_path: String, file_name: String) -> Result<(), String> {
    modrinth_service::enable_mod(profile_path, file_name)
}

#[tauri::command]
pub fn disable_mod(profile_path: String, file_name: String) -> Result<(), String> {
    modrinth_service::disable_mod(profile_path, file_name)
}

#[tauri::command]
pub fn check_mod_updates(profile_path: String) -> Result<Vec<String>, String> {
    modrinth_service::check_mod_updates(profile_path)
}

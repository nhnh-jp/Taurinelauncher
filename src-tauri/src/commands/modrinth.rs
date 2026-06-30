use serde::Serialize;

use crate::{models::mod_info::ModInfo, services::modrinth_service};

#[derive(Debug, Clone, Serialize)]
pub struct ModrinthSearchResult {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub icon_url: String,
    pub downloads: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModrinthVersionResult {
    pub version_id: String,
    pub name: String,
    pub version_number: String,
    pub file_name: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModUpdateResult {
    pub name: String,
    pub file_name: String,
    pub current_version_id: String,
    pub latest_version_id: String,
    pub latest_version_number: String,
    pub latest_file_name: String,
}

#[tauri::command]
pub fn list_installed_mods(profile_path: String) -> Result<Vec<ModInfo>, String> {
    modrinth_service::list_installed_mods(profile_path)
}

#[tauri::command]
pub fn search_modrinth(
    query: String,
    version: String,
    loader: String,
) -> Result<Vec<ModrinthSearchResult>, String> {
    modrinth_service::search_modrinth(query, version, loader)
}

#[tauri::command]
pub fn get_modrinth_versions(
    project_id: String,
    version: String,
    loader: String,
) -> Result<Vec<ModrinthVersionResult>, String> {
    modrinth_service::get_modrinth_versions(project_id, version, loader)
}

#[tauri::command]
pub fn install_mod(
    profile_path: String,
    project_id: String,
    version_id: String,
) -> Result<ModInfo, String> {
    modrinth_service::install_mod(profile_path, project_id, version_id)
}

#[tauri::command]
pub fn update_mod(
    profile_path: String,
    file_name: String,
    version_id: String,
) -> Result<ModInfo, String> {
    modrinth_service::update_mod(profile_path, file_name, version_id)
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
pub fn check_mod_updates(profile_path: String) -> Result<Vec<ModUpdateResult>, String> {
    modrinth_service::check_mod_updates(profile_path)
}

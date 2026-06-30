use serde::Serialize;

use crate::services::modrinth_service;

#[derive(Debug, Clone, Serialize)]
pub struct ModrinthSearchResult { pub project_id: String, pub title: String, pub description: String }

#[tauri::command]
pub fn search_modrinth(_query: String, _version: String, _loader: String) -> Result<Vec<ModrinthSearchResult>, String> { modrinth_service::phase_two() }

#[tauri::command]
pub fn get_modrinth_versions(_project_id: String, _version: String, _loader: String) -> Result<Vec<String>, String> { modrinth_service::phase_two() }

#[tauri::command]
pub fn install_mod(_profile_path: String, _project_id: String, _version_id: String) -> Result<(), String> { modrinth_service::phase_two() }

#[tauri::command]
pub fn remove_mod(_profile_path: String, _file_name: String) -> Result<(), String> { modrinth_service::phase_two() }

#[tauri::command]
pub fn enable_mod(_profile_path: String, _file_name: String) -> Result<(), String> { modrinth_service::phase_two() }

#[tauri::command]
pub fn disable_mod(_profile_path: String, _file_name: String) -> Result<(), String> { modrinth_service::phase_two() }

#[tauri::command]
pub fn check_mod_updates(_profile_path: String) -> Result<Vec<String>, String> { modrinth_service::phase_two() }

use crate::services::profile_service;

#[tauri::command]
pub fn ensure_data_dirs() -> Result<(), String> {
    profile_service::ensure_base_dirs()
}

use crate::{models::profile::{Profile, ProfileSummary}, services::profile_service};

#[tauri::command]
pub fn create_profile(version: String, loader: String, name: String, loader_version: String, auto_memory: bool) -> Result<ProfileSummary, String> {
    profile_service::create_profile(version, loader, name, loader_version, auto_memory)
}

#[tauri::command]
pub fn list_profiles() -> Result<Vec<ProfileSummary>, String> { profile_service::list_profiles() }

#[tauri::command]
pub fn read_profile(profile_path: String) -> Result<Profile, String> { profile_service::read_profile(profile_path) }

#[tauri::command]
pub fn update_profile(profile_path: String, profile: Profile) -> Result<ProfileSummary, String> { profile_service::update_profile(profile_path, profile) }

#[tauri::command]
pub fn delete_profile(profile_path: String) -> Result<(), String> { profile_service::delete_profile(profile_path) }

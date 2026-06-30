use crate::services::memory_service::{self, MemoryPlan};

#[tauri::command]
pub fn calculate_memory(profile_path: String) -> Result<MemoryPlan, String> { memory_service::calculate_memory(profile_path) }

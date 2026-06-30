use sysinfo::System;

use crate::services::profile_service;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryPlan {
    pub total_memory_mb: u64,
    pub os_reserved_mb: u64,
    pub mod_count: usize,
    pub recommended_mb: u64,
    pub capped_by_system: bool,
    pub manual_override: bool,
}

pub fn calculate_memory(profile_path: String) -> Result<MemoryPlan, String> {
    let profile = profile_service::read_profile(profile_path.clone())?;
    let summary = profile_service::summarize_profile(&profile_service::resolve_profile_path(&profile_path)?)?;
    let mut system = System::new();
    system.refresh_memory();
    let total_memory_mb = system.total_memory() / 1024 / 1024;
    let os_reserved_mb = 2048;

    if !profile.launch.auto_memory {
        return Ok(MemoryPlan { total_memory_mb, os_reserved_mb, mod_count: summary.mod_count, recommended_mb: profile.launch.memory_max_mb as u64, capped_by_system: false, manual_override: true });
    }

    let target = match summary.mod_count {
        0..=20 => 2048,
        21..=80 => 4096,
        81..=150 => 6144,
        _ => 8192,
    };
    let available = total_memory_mb.saturating_sub(os_reserved_mb).max(profile.launch.memory_min_mb as u64);
    let recommended_mb = target.min(available).max(profile.launch.memory_min_mb as u64);
    Ok(MemoryPlan { total_memory_mb, os_reserved_mb, mod_count: summary.mod_count, recommended_mb, capped_by_system: recommended_mb < target, manual_override: false })
}

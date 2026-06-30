use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: String,
    pub launch: LaunchSettings,
    pub game: GameSettings,
    pub mods: ModSettings,
    pub server: ServerSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchSettings {
    pub auto_memory: bool,
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub java_path: String,
    pub extra_jvm_args: Vec<String>,
    pub extra_game_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub fullscreen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSettings {
    pub check_updates_on_start: bool,
    pub auto_install_dependencies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub enabled: bool,
    pub name: String,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileSummary {
    pub name: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: String,
    pub path: String,
    pub mod_count: usize,
    pub enabled_mod_count: usize,
    pub disabled_mod_count: usize,
    pub auto_memory: bool,
    pub memory_max_mb: u32,
    pub server_enabled: bool,
}

impl Profile {
    pub fn new(name: String, minecraft_version: String, loader: String, loader_version: String, auto_memory: bool) -> Self {
        Self {
            name,
            minecraft_version,
            loader,
            loader_version,
            launch: LaunchSettings { auto_memory, memory_min_mb: 512, memory_max_mb: 4096, java_path: "auto".to_string(), extra_jvm_args: vec![], extra_game_args: vec![] },
            game: GameSettings { resolution_width: 1280, resolution_height: 720, fullscreen: false },
            mods: ModSettings { check_updates_on_start: true, auto_install_dependencies: true },
            server: ServerSettings { enabled: false, name: String::new(), address: String::new(), port: 25565 },
        }
    }
}

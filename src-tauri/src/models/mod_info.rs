use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModIndex {
    pub schema_version: u32,
    pub mods: Vec<ModInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub name: String,
    pub project_id: String,
    pub version_id: String,
    pub file_name: String,
    pub sha512: String,
    pub enabled: bool,
    pub source: String,
    pub minecraft_version: String,
    pub loader: String,
}

impl Default for ModIndex {
    fn default() -> Self {
        Self { schema_version: 1, mods: vec![] }
    }
}

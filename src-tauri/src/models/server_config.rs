use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub description: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: String,
    pub address: String,
    pub port: u16,
    pub mods: Vec<ServerMod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMod {
    pub name: String,
    pub source: String,
    pub project_id: String,
    pub required: bool,
}

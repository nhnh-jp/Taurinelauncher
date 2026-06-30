use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{
    models::launch_config::LaunchResult,
    services::{java_service, profile_service},
};

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const USER_AGENT: &str = "TaurineLauncher/0.1.0 (github.com/nhnh-jp/Taurinelauncher)";

#[derive(Debug, Deserialize)]
struct VersionManifest {
    versions: Vec<VersionManifestEntry>,
}

#[derive(Debug, Deserialize)]
struct VersionManifestEntry {
    id: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct VersionJson {
    #[serde(rename = "mainClass")]
    main_class: String,
    downloads: VersionDownloads,
}

#[derive(Debug, Deserialize)]
struct VersionDownloads {
    client: DownloadInfo,
}

#[derive(Debug, Deserialize)]
struct DownloadInfo {
    url: String,
}

pub fn launch_minecraft(profile_path: String) -> Result<LaunchResult, String> {
    let profile = profile_service::read_profile(profile_path.clone())?;
    let java = java_service::detect_java()?;
    if !java.found {
        return Err(
            "Java was not found. Install Java or set JAVA_HOME before launching Minecraft."
                .to_string(),
        );
    }

    let profile_dir = profile_service::resolve_profile_path(&profile_path)?;
    let runtime = prepare_minecraft_runtime(&profile.minecraft_version)?;
    Ok(LaunchResult {
        command_preview: format!(
            "{} -Xmx{}M -cp {} {} --version {} --gameDir {}",
            java.path,
            profile.launch.memory_max_mb,
            runtime.client_jar.to_string_lossy(),
            runtime.main_class,
            profile.minecraft_version,
            profile_dir.to_string_lossy()
        ),
        log_path: profile_dir
            .join("logs")
            .join("latest.log")
            .to_string_lossy()
            .to_string(),
    })
}

pub fn stream_logs(_profile_path: String) -> Result<Vec<String>, String> {
    Ok(vec![
        "Minecraft log streaming is not connected yet.".to_string()
    ])
}

struct PreparedRuntime {
    client_jar: PathBuf,
    main_class: String,
}

fn prepare_minecraft_runtime(version: &str) -> Result<PreparedRuntime, String> {
    let root = profile_service::data_dir()?
        .join("runtime")
        .join("minecraft")
        .join("versions")
        .join(version);
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;

    let version_json_path = root.join(format!("{}.json", version));
    let version_json_text = if version_json_path.exists() {
        fs::read_to_string(&version_json_path).map_err(|error| error.to_string())?
    } else {
        let manifest: VersionManifest = get_json(VERSION_MANIFEST_URL)?;
        let entry = manifest
            .versions
            .into_iter()
            .find(|entry| entry.id == version)
            .ok_or_else(|| {
                format!(
                    "Minecraft version {} was not found in Mojang manifest",
                    version
                )
            })?;
        let text = get_text(&entry.url)?;
        fs::write(&version_json_path, &text).map_err(|error| error.to_string())?;
        text
    };

    let version_json: VersionJson =
        serde_json::from_str(&version_json_text).map_err(|error| error.to_string())?;
    let client_jar = root.join(format!("{}.jar", version));
    if !client_jar.exists() {
        download_file(&version_json.downloads.client.url, &client_jar)?;
    }

    Ok(PreparedRuntime {
        client_jar,
        main_class: version_json.main_class,
    })
}

fn client() -> Result<Client, String> {
    Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|error| error.to_string())
}

fn get_json<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T, String> {
    client()?
        .get(url)
        .send()
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .json()
        .map_err(|error| error.to_string())
}

fn get_text(url: &str) -> Result<String, String> {
    client()?
        .get(url)
        .send()
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .text()
        .map_err(|error| error.to_string())
}

fn download_file(url: &str, target: &Path) -> Result<(), String> {
    let bytes = client()?
        .get(url)
        .send()
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .bytes()
        .map_err(|error| error.to_string())?;
    let mut output = fs::File::create(target).map_err(|error| error.to_string())?;
    output.write_all(&bytes).map_err(|error| error.to_string())
}

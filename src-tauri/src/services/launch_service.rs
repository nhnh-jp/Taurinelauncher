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
    #[serde(default)]
    libraries: Vec<Library>,
    #[serde(rename = "assetIndex")]
    asset_index: Option<AssetIndex>,
}

#[derive(Debug, Deserialize)]
struct VersionDownloads {
    client: DownloadInfo,
}

#[derive(Debug, Deserialize)]
struct Library {
    downloads: Option<LibraryDownloads>,
}

#[derive(Debug, Deserialize)]
struct LibraryDownloads {
    artifact: Option<LibraryArtifact>,
}

#[derive(Debug, Deserialize)]
struct LibraryArtifact {
    path: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct AssetIndex {
    id: String,
    url: String,
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
    let classpath = runtime_classpath(&runtime);
    Ok(LaunchResult {
        command_preview: format!(
            "{} -Xmx{}M -cp {} {} --version {} --gameDir {} --assetsDir {} --assetIndex {}",
            java.path,
            profile.launch.memory_max_mb,
            classpath,
            runtime.main_class,
            profile.minecraft_version,
            profile_dir.to_string_lossy(),
            runtime.assets_dir.to_string_lossy(),
            runtime.asset_index_id
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
    library_jars: Vec<PathBuf>,
    assets_dir: PathBuf,
    asset_index_id: String,
    main_class: String,
}

fn prepare_minecraft_runtime(version: &str) -> Result<PreparedRuntime, String> {
    let data_dir = profile_service::data_dir()?;
    let minecraft_root = data_dir.join("runtime").join("minecraft");
    let version_root = minecraft_root.join("versions").join(version);
    let libraries_root = minecraft_root.join("libraries");
    let assets_dir = minecraft_root.join("assets");
    let asset_indexes_dir = assets_dir.join("indexes");

    fs::create_dir_all(&version_root).map_err(|error| error.to_string())?;
    fs::create_dir_all(&libraries_root).map_err(|error| error.to_string())?;
    fs::create_dir_all(&asset_indexes_dir).map_err(|error| error.to_string())?;

    let version_json_path = version_root.join(format!("{}.json", version));
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
    let client_jar = version_root.join(format!("{}.jar", version));
    if !client_jar.exists() {
        download_file(&version_json.downloads.client.url, &client_jar)?;
    }

    let library_jars = prepare_libraries(&version_json, &libraries_root)?;
    let asset_index_id = prepare_asset_index(&version_json, &asset_indexes_dir)?;

    Ok(PreparedRuntime {
        client_jar,
        library_jars,
        assets_dir,
        asset_index_id,
        main_class: version_json.main_class,
    })
}

fn prepare_libraries(
    version_json: &VersionJson,
    libraries_root: &Path,
) -> Result<Vec<PathBuf>, String> {
    let mut jars = Vec::new();
    for library in &version_json.libraries {
        let Some(downloads) = &library.downloads else {
            continue;
        };
        let Some(artifact) = &downloads.artifact else {
            continue;
        };
        let target = libraries_root.join(path_from_manifest(&artifact.path)?);
        if !target.exists() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            download_file(&artifact.url, &target)?;
        }
        jars.push(target);
    }
    Ok(jars)
}

fn prepare_asset_index(
    version_json: &VersionJson,
    asset_indexes_dir: &Path,
) -> Result<String, String> {
    let Some(asset_index) = &version_json.asset_index else {
        return Ok(String::new());
    };
    let target = asset_indexes_dir.join(format!("{}.json", asset_index.id));
    if !target.exists() {
        let text = get_text(&asset_index.url)?;
        fs::write(&target, text).map_err(|error| error.to_string())?;
    }
    Ok(asset_index.id.clone())
}

fn runtime_classpath(runtime: &PreparedRuntime) -> String {
    let separator = if cfg!(windows) { ";" } else { ":" };
    runtime
        .library_jars
        .iter()
        .chain(std::iter::once(&runtime.client_jar))
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(separator)
}

fn path_from_manifest(path: &str) -> Result<PathBuf, String> {
    if path.is_empty()
        || path.contains("..")
        || path.contains('\\')
        || path.split('/').any(|part| part.is_empty())
    {
        return Err("invalid path in Minecraft version manifest".to_string());
    }
    Ok(path.split('/').collect())
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

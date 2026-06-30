use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use reqwest::blocking::Client;
use serde::Deserialize;
use zip::ZipArchive;

use crate::{
    models::{launch_config::LaunchResult, profile::Profile},
    services::{java_service, profile_service},
};

const VERSION_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const ASSET_OBJECT_BASE_URL: &str = "https://resources.download.minecraft.net";
const USER_AGENT: &str = "TaurineLauncher/0.1.0 (github.com/nhnh-jp/Taurinelauncher)";
const DEV_USERNAME: &str = "Player";
const DEV_UUID: &str = "00000000-0000-0000-0000-000000000000";
const DEV_ACCESS_TOKEN: &str = "0";

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
    #[serde(default, rename = "minecraftArguments")]
    minecraft_arguments: Option<String>,
    #[serde(default)]
    arguments: Option<VersionArguments>,
}

#[derive(Debug, Deserialize)]
struct VersionArguments {
    #[serde(default)]
    game: Vec<ArgumentEntry>,
    #[serde(default)]
    jvm: Vec<ArgumentEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ArgumentEntry {
    Plain(String),
    Ruled {
        rules: Vec<LibraryRule>,
        value: ArgumentValue,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ArgumentValue {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Deserialize)]
struct VersionDownloads {
    client: DownloadInfo,
}

#[derive(Debug, Deserialize)]
struct Library {
    #[serde(default)]
    downloads: Option<LibraryDownloads>,
    #[serde(default)]
    natives: HashMap<String, String>,
    #[serde(default)]
    rules: Vec<LibraryRule>,
}

#[derive(Debug, Deserialize)]
struct LibraryDownloads {
    artifact: Option<LibraryArtifact>,
    #[serde(default)]
    classifiers: HashMap<String, LibraryArtifact>,
}

#[derive(Debug, Deserialize)]
struct LibraryArtifact {
    path: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct LibraryRule {
    action: String,
    os: Option<LibraryRuleOs>,
}

#[derive(Debug, Deserialize)]
struct LibraryRuleOs {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AssetIndex {
    id: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct AssetObjects {
    objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
struct AssetObject {
    hash: String,
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
    let command = build_launch_command(&java.path, &profile, &profile_dir, &runtime);
    Ok(LaunchResult {
        command_preview: command.join(" "),
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
    natives_dir: PathBuf,
    assets_dir: PathBuf,
    asset_index_id: String,
    main_class: String,
    game_args: Vec<String>,
    jvm_args: Vec<String>,
}

fn prepare_minecraft_runtime(version: &str) -> Result<PreparedRuntime, String> {
    let data_dir = profile_service::data_dir()?;
    let minecraft_root = data_dir.join("runtime").join("minecraft");
    let version_root = minecraft_root.join("versions").join(version);
    let libraries_root = minecraft_root.join("libraries");
    let natives_root = minecraft_root.join("natives").join(version);
    let assets_dir = minecraft_root.join("assets");
    let asset_indexes_dir = assets_dir.join("indexes");
    let asset_objects_dir = assets_dir.join("objects");

    fs::create_dir_all(&version_root).map_err(|error| error.to_string())?;
    fs::create_dir_all(&libraries_root).map_err(|error| error.to_string())?;
    fs::create_dir_all(&natives_root).map_err(|error| error.to_string())?;
    fs::create_dir_all(&asset_indexes_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&asset_objects_dir).map_err(|error| error.to_string())?;

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
    prepare_natives(&version_json, &libraries_root, &natives_root)?;
    let asset_index_id = prepare_asset_index(&version_json, &asset_indexes_dir)?;
    if !asset_index_id.is_empty() {
        let asset_index_path = asset_indexes_dir.join(format!("{}.json", asset_index_id));
        prepare_asset_objects(&asset_index_path, &asset_objects_dir)?;
    }
    let (game_args, jvm_args) = version_arguments(&version_json);

    Ok(PreparedRuntime {
        client_jar,
        library_jars,
        natives_dir: natives_root,
        assets_dir,
        asset_index_id,
        main_class: version_json.main_class,
        game_args,
        jvm_args,
    })
}

fn build_launch_command(
    java_path: &str,
    profile: &Profile,
    profile_dir: &Path,
    runtime: &PreparedRuntime,
) -> Vec<String> {
    let classpath = runtime_classpath(runtime);
    let mut replacements = HashMap::new();
    replacements.insert("auth_player_name", DEV_USERNAME.to_string());
    replacements.insert("version_name", profile.minecraft_version.clone());
    replacements.insert("game_directory", profile_dir.to_string_lossy().to_string());
    replacements.insert(
        "assets_root",
        runtime.assets_dir.to_string_lossy().to_string(),
    );
    replacements.insert("assets_index_name", runtime.asset_index_id.clone());
    replacements.insert("auth_uuid", DEV_UUID.to_string());
    replacements.insert("auth_access_token", DEV_ACCESS_TOKEN.to_string());
    replacements.insert("user_type", "legacy".to_string());
    replacements.insert("version_type", "Taurine".to_string());
    replacements.insert(
        "natives_directory",
        runtime.natives_dir.to_string_lossy().to_string(),
    );
    replacements.insert("launcher_name", "TaurineLauncher".to_string());
    replacements.insert("launcher_version", "0.1.0".to_string());
    replacements.insert("classpath", classpath);

    let mut command = vec![java_path.to_string()];
    command.push(format!("-Xms{}M", profile.launch.memory_min_mb));
    command.push(format!("-Xmx{}M", profile.launch.memory_max_mb));
    command.extend(profile.launch.extra_jvm_args.iter().cloned());
    command.extend(resolve_arguments(&runtime.jvm_args, &replacements));
    if runtime.jvm_args.is_empty() {
        command.push(format!(
            "-Djava.library.path={}",
            runtime.natives_dir.to_string_lossy()
        ));
        command.push("-cp".to_string());
        command.push(replacements["classpath"].clone());
    }
    command.push(runtime.main_class.clone());
    command.extend(resolve_arguments(&runtime.game_args, &replacements));
    command.extend(profile.launch.extra_game_args.iter().cloned());
    command
}

fn version_arguments(version_json: &VersionJson) -> (Vec<String>, Vec<String>) {
    if let Some(arguments) = &version_json.arguments {
        return (
            flatten_arguments(&arguments.game),
            flatten_arguments(&arguments.jvm),
        );
    }
    let game_args = version_json
        .minecraft_arguments
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .map(ToString::to_string)
        .collect();
    (game_args, vec![])
}

fn flatten_arguments(entries: &[ArgumentEntry]) -> Vec<String> {
    let mut values = Vec::new();
    for entry in entries {
        match entry {
            ArgumentEntry::Plain(value) => values.push(value.clone()),
            ArgumentEntry::Ruled { rules, value } => {
                if rules.iter().any(rule_matches_current_os) {
                    match value {
                        ArgumentValue::One(item) => values.push(item.clone()),
                        ArgumentValue::Many(items) => values.extend(items.iter().cloned()),
                    }
                }
            }
        }
    }
    values
}

fn resolve_arguments(args: &[String], replacements: &HashMap<&str, String>) -> Vec<String> {
    args.iter()
        .map(|arg| {
            let mut resolved = arg.clone();
            for (key, value) in replacements {
                resolved = resolved.replace(&format!("${{{}}}", key), value);
            }
            resolved
        })
        .collect()
}

fn prepare_libraries(
    version_json: &VersionJson,
    libraries_root: &Path,
) -> Result<Vec<PathBuf>, String> {
    let mut jars = Vec::new();
    for library in version_json
        .libraries
        .iter()
        .filter(|library| should_use_library(library))
    {
        let Some(downloads) = &library.downloads else {
            continue;
        };
        let Some(artifact) = &downloads.artifact else {
            continue;
        };
        let target = libraries_root.join(path_from_manifest(&artifact.path)?);
        if !target.exists() {
            download_file(&artifact.url, &target)?;
        }
        jars.push(target);
    }
    Ok(jars)
}

fn prepare_natives(
    version_json: &VersionJson,
    libraries_root: &Path,
    natives_root: &Path,
) -> Result<(), String> {
    for library in version_json
        .libraries
        .iter()
        .filter(|library| should_use_library(library))
    {
        let Some(classifier_name) = native_classifier(library) else {
            continue;
        };
        let Some(downloads) = &library.downloads else {
            continue;
        };
        let Some(artifact) = downloads.classifiers.get(&classifier_name) else {
            continue;
        };
        let native_jar = libraries_root.join(path_from_manifest(&artifact.path)?);
        if !native_jar.exists() {
            download_file(&artifact.url, &native_jar)?;
        }
        extract_native_jar(&native_jar, natives_root)?;
    }
    Ok(())
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

fn prepare_asset_objects(asset_index_path: &Path, asset_objects_dir: &Path) -> Result<(), String> {
    let text = fs::read_to_string(asset_index_path).map_err(|error| error.to_string())?;
    let index: AssetObjects = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    for object in index.objects.values() {
        let prefix = object
            .hash
            .get(0..2)
            .ok_or_else(|| "invalid asset hash in Minecraft asset index".to_string())?;
        let target = asset_objects_dir.join(prefix).join(&object.hash);
        if target.exists() {
            continue;
        }
        let url = format!("{}/{}/{}", ASSET_OBJECT_BASE_URL, prefix, object.hash);
        download_file(&url, &target)?;
    }
    Ok(())
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

fn native_classifier(library: &Library) -> Option<String> {
    library
        .natives
        .get(current_os_name())
        .map(|classifier| classifier.replace("${arch}", current_arch_bits()))
}

fn should_use_library(library: &Library) -> bool {
    if library.rules.is_empty() {
        return true;
    }
    let mut allowed = false;
    for rule in &library.rules {
        if rule_matches_current_os(rule) {
            allowed = rule.action == "allow";
        }
    }
    allowed
}

fn rule_matches_current_os(rule: &LibraryRule) -> bool {
    match &rule.os {
        Some(os) => os
            .name
            .as_deref()
            .is_none_or(|name| name == current_os_name()),
        None => true,
    }
}

fn current_os_name() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

fn current_arch_bits() -> &'static str {
    if cfg!(target_pointer_width = "64") {
        "64"
    } else {
        "32"
    }
}

fn extract_native_jar(native_jar: &Path, natives_root: &Path) -> Result<(), String> {
    let file = fs::File::open(native_jar).map_err(|error| error.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        if entry.is_dir() || entry.name().starts_with("META-INF/") {
            continue;
        }
        let Some(enclosed_path) = entry.enclosed_name() else {
            continue;
        };
        let target = natives_root.join(enclosed_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut output = fs::File::create(&target).map_err(|error| error.to_string())?;
        io::copy(&mut entry, &mut output).map_err(|error| error.to_string())?;
    }
    Ok(())
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
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
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

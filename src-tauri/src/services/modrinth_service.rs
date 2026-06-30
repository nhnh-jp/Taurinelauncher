use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{
    commands::modrinth::{ModUpdateResult, ModrinthSearchResult, ModrinthVersionResult},
    models::{
        mod_info::{ModIndex, ModInfo},
        profile::Profile,
    },
    services::profile_service,
};

const API_BASE: &str = "https://api.modrinth.com/v2";
const USER_AGENT: &str = "TaurineLauncher/0.1.0 (github.com/nhnh-jp/Taurinelauncher)";

#[derive(Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<SearchHit>,
}

#[derive(Debug, Deserialize)]
struct SearchHit {
    project_id: String,
    title: String,
    description: String,
    #[serde(default)]
    icon_url: Option<String>,
    #[serde(default)]
    downloads: u64,
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    id: String,
    name: String,
    version_number: String,
    project_id: String,
    #[serde(default)]
    game_versions: Vec<String>,
    #[serde(default)]
    loaders: Vec<String>,
    files: Vec<VersionFile>,
}

#[derive(Debug, Deserialize)]
struct VersionFile {
    filename: String,
    url: String,
    #[serde(default)]
    primary: bool,
    #[serde(default)]
    hashes: FileHashes,
}

#[derive(Debug, Default, Deserialize)]
struct FileHashes {
    #[serde(default)]
    sha512: String,
}

pub fn search_modrinth(
    query: String,
    version: String,
    loader: String,
) -> Result<Vec<ModrinthSearchResult>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(vec![]);
    }

    let facets = serde_json::json!([
        ["project_type:mod"],
        [format!("versions:{}", version)],
        [format!("categories:{}", loader)]
    ]);
    let url = format!(
        "{}/search?query={}&limit=12&index=relevance&facets={}",
        API_BASE,
        urlencoding::encode(query),
        urlencoding::encode(&facets.to_string())
    );
    let response: SearchResponse = client()?
        .get(url)
        .send()
        .map_err(request_error)?
        .error_for_status()
        .map_err(request_error)?
        .json()
        .map_err(request_error)?;

    Ok(response
        .hits
        .into_iter()
        .map(|hit| ModrinthSearchResult {
            project_id: hit.project_id,
            title: hit.title,
            description: hit.description,
            icon_url: hit.icon_url.unwrap_or_default(),
            downloads: hit.downloads,
        })
        .collect())
}

pub fn get_modrinth_versions(
    project_id: String,
    version: String,
    loader: String,
) -> Result<Vec<ModrinthVersionResult>, String> {
    let versions = fetch_project_versions(&project_id)?;
    Ok(versions
        .into_iter()
        .filter(|item| {
            item.game_versions
                .iter()
                .any(|game_version| game_version == &version)
        })
        .filter(|item| {
            item.loaders
                .iter()
                .any(|item_loader| item_loader == &loader)
        })
        .filter_map(|item| {
            let file = primary_file(&item.files)?;
            Some(ModrinthVersionResult {
                version_id: item.id,
                name: item.name,
                version_number: item.version_number,
                file_name: file.filename.clone(),
                download_url: file.url.clone(),
            })
        })
        .collect())
}

pub fn install_mod(
    profile_path: String,
    project_id: String,
    version_id: String,
) -> Result<ModInfo, String> {
    let profile_dir = profile_service::resolve_profile_path(&profile_path)?;
    let profile: Profile = profile_service::read_profile(profile_path)?;
    let version = fetch_project_versions(&project_id)?
        .into_iter()
        .find(|item| item.id == version_id)
        .ok_or_else(|| "selected Modrinth version was not found".to_string())?;
    let file = primary_file(&version.files)
        .ok_or_else(|| "selected Modrinth version has no downloadable jar file".to_string())?;
    let file_name = ensure_safe_file_name(&file.filename)?;
    let target = profile_dir.join("mods").join(&file_name);
    if target.exists() || profile_dir.join("disabled-mods").join(&file_name).exists() {
        return Err("a mod with the same file name already exists in this profile".to_string());
    }

    fs::create_dir_all(profile_dir.join("mods")).map_err(|error| error.to_string())?;
    download_file(&file.url, &target)?;

    let mut index = read_index(&profile_dir)?;
    let mod_info = ModInfo {
        name: version.name,
        project_id: version.project_id,
        version_id: version.id,
        file_name,
        sha512: file.hashes.sha512.clone(),
        enabled: true,
        source: "modrinth".to_string(),
        minecraft_version: profile.minecraft_version,
        loader: profile.loader,
    };
    index
        .mods
        .retain(|item| item.file_name != mod_info.file_name);
    index.mods.push(mod_info.clone());
    sync_index_with_files(&profile_dir, &mut index)?;
    write_index(&profile_dir, &index)?;
    Ok(mod_info)
}

pub fn update_mod(
    profile_path: String,
    file_name: String,
    version_id: String,
) -> Result<ModInfo, String> {
    let old_file_name = ensure_safe_file_name(&file_name)?;
    let profile_dir = profile_service::resolve_profile_path(&profile_path)?;
    let profile: Profile = profile_service::read_profile(profile_path)?;
    let mut index = read_index(&profile_dir)?;
    sync_index_with_files(&profile_dir, &mut index)?;

    let existing = index
        .mods
        .iter()
        .find(|item| item.file_name == old_file_name)
        .cloned()
        .ok_or_else(|| "target mod was not found in index.json".to_string())?;
    if existing.source != "modrinth" || existing.project_id.is_empty() {
        return Err("only Modrinth-managed mods can be updated".to_string());
    }

    let old_enabled_path = profile_dir.join("mods").join(&old_file_name);
    let old_disabled_path = profile_dir.join("disabled-mods").join(&old_file_name);
    let enabled = old_enabled_path.exists();
    let old_path = if enabled {
        old_enabled_path
    } else if old_disabled_path.exists() {
        old_disabled_path
    } else {
        return Err("target mod file was not found".to_string());
    };

    let version = fetch_project_versions(&existing.project_id)?
        .into_iter()
        .find(|item| item.id == version_id)
        .ok_or_else(|| "selected Modrinth version was not found".to_string())?;
    let file = primary_file(&version.files)
        .ok_or_else(|| "selected Modrinth version has no downloadable jar file".to_string())?;
    let new_file_name = ensure_safe_file_name(&file.filename)?;
    let target_dir = if enabled { "mods" } else { "disabled-mods" };
    let target_path = profile_dir.join(target_dir).join(&new_file_name);
    if new_file_name != old_file_name && target_path.exists() {
        return Err("a mod with the updated file name already exists in this profile".to_string());
    }

    fs::create_dir_all(profile_dir.join(target_dir)).map_err(|error| error.to_string())?;
    let temp_path = profile_dir
        .join(target_dir)
        .join(format!(".download-{}", new_file_name));
    remove_if_exists(temp_path.clone())?;
    download_file(&file.url, &temp_path)?;
    fs::remove_file(&old_path).map_err(|error| error.to_string())?;
    fs::rename(&temp_path, &target_path).map_err(|error| error.to_string())?;

    let updated = ModInfo {
        name: version.name,
        project_id: version.project_id,
        version_id: version.id,
        file_name: new_file_name,
        sha512: file.hashes.sha512.clone(),
        enabled,
        source: "modrinth".to_string(),
        minecraft_version: profile.minecraft_version,
        loader: profile.loader,
    };
    index.mods.retain(|item| item.file_name != old_file_name);
    index.mods.push(updated.clone());
    sync_index_with_files(&profile_dir, &mut index)?;
    write_index(&profile_dir, &index)?;
    Ok(updated)
}
pub fn list_installed_mods(profile_path: String) -> Result<Vec<ModInfo>, String> {
    let profile_dir = profile_service::resolve_profile_path(&profile_path)?;
    let mut index = read_index(&profile_dir)?;
    sync_index_with_files(&profile_dir, &mut index)?;
    write_index(&profile_dir, &index)?;
    index.mods.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(index.mods)
}

pub fn remove_mod(profile_path: String, file_name: String) -> Result<(), String> {
    let safe_name = ensure_safe_file_name(&file_name)?;
    let profile_dir = profile_service::resolve_profile_path(&profile_path)?;
    remove_if_exists(profile_dir.join("mods").join(&safe_name))?;
    remove_if_exists(profile_dir.join("disabled-mods").join(&safe_name))?;

    let mut index = read_index(&profile_dir)?;
    index
        .mods
        .retain(|mod_info| mod_info.file_name != safe_name);
    write_index(&profile_dir, &index)
}

pub fn enable_mod(profile_path: String, file_name: String) -> Result<(), String> {
    move_mod(profile_path, file_name, false)
}

pub fn disable_mod(profile_path: String, file_name: String) -> Result<(), String> {
    move_mod(profile_path, file_name, true)
}

pub fn check_mod_updates(profile_path: String) -> Result<Vec<ModUpdateResult>, String> {
    let profile_dir = profile_service::resolve_profile_path(&profile_path)?;
    let profile: Profile = profile_service::read_profile(profile_path)?;
    let mut index = read_index(&profile_dir)?;
    sync_index_with_files(&profile_dir, &mut index)?;
    write_index(&profile_dir, &index)?;

    let mut updates = Vec::new();
    for mod_info in index.mods.iter().filter(|item| item.source == "modrinth") {
        if mod_info.project_id.is_empty() || mod_info.version_id.is_empty() {
            continue;
        }
        let Some(latest) = latest_compatible_version(
            &mod_info.project_id,
            &profile.minecraft_version,
            &profile.loader,
        )?
        else {
            continue;
        };
        if latest.id == mod_info.version_id {
            continue;
        }
        let Some(file) = primary_file(&latest.files) else {
            continue;
        };
        updates.push(ModUpdateResult {
            name: mod_info.name.clone(),
            file_name: mod_info.file_name.clone(),
            current_version_id: mod_info.version_id.clone(),
            latest_version_id: latest.id,
            latest_version_number: latest.version_number,
            latest_file_name: file.filename.clone(),
        });
    }
    Ok(updates)
}

fn download_file(url: &str, target: &Path) -> Result<(), String> {
    let bytes = client()?
        .get(url)
        .send()
        .map_err(request_error)?
        .error_for_status()
        .map_err(request_error)?
        .bytes()
        .map_err(request_error)?;
    let mut output = fs::File::create(target).map_err(|error| error.to_string())?;
    output.write_all(&bytes).map_err(|error| error.to_string())
}
fn latest_compatible_version(
    project_id: &str,
    minecraft_version: &str,
    loader: &str,
) -> Result<Option<VersionResponse>, String> {
    Ok(fetch_project_versions(project_id)?
        .into_iter()
        .find(|item| {
            item.game_versions
                .iter()
                .any(|game_version| game_version == minecraft_version)
                && item.loaders.iter().any(|item_loader| item_loader == loader)
                && primary_file(&item.files).is_some()
        }))
}
fn fetch_project_versions(project_id: &str) -> Result<Vec<VersionResponse>, String> {
    let url = format!(
        "{}/project/{}/version",
        API_BASE,
        urlencoding::encode(project_id)
    );
    client()?
        .get(url)
        .send()
        .map_err(request_error)?
        .error_for_status()
        .map_err(request_error)?
        .json()
        .map_err(request_error)
}

fn client() -> Result<Client, String> {
    Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(request_error)
}

fn primary_file(files: &[VersionFile]) -> Option<&VersionFile> {
    files
        .iter()
        .find(|file| file.primary && file.filename.to_ascii_lowercase().ends_with(".jar"))
        .or_else(|| {
            files
                .iter()
                .find(|file| file.filename.to_ascii_lowercase().ends_with(".jar"))
        })
}

fn request_error(error: reqwest::Error) -> String {
    error.to_string()
}

fn move_mod(profile_path: String, file_name: String, disable: bool) -> Result<(), String> {
    let safe_name = ensure_safe_file_name(&file_name)?;
    let profile_dir = profile_service::resolve_profile_path(&profile_path)?;
    let from_dir = if disable { "mods" } else { "disabled-mods" };
    let to_dir = if disable { "disabled-mods" } else { "mods" };
    let from = profile_dir.join(from_dir).join(&safe_name);
    let to = profile_dir.join(to_dir).join(&safe_name);

    if !from.exists() {
        return Err(format!("target mod was not found in {}", from_dir));
    }
    if to.exists() {
        return Err(format!(
            "a mod with the same file name already exists in {}",
            to_dir
        ));
    }

    fs::create_dir_all(profile_dir.join(to_dir)).map_err(|error| error.to_string())?;
    fs::rename(from, to).map_err(|error| error.to_string())?;

    let mut index = read_index(&profile_dir)?;
    sync_index_with_files(&profile_dir, &mut index)?;
    if let Some(mod_info) = index
        .mods
        .iter_mut()
        .find(|mod_info| mod_info.file_name == safe_name)
    {
        mod_info.enabled = !disable;
    }
    write_index(&profile_dir, &index)
}

fn read_index(profile_dir: &Path) -> Result<ModIndex, String> {
    let path = profile_dir.join("index.json");
    if !path.exists() {
        return Ok(ModIndex::default());
    }
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn write_index(profile_dir: &Path, index: &ModIndex) -> Result<(), String> {
    let text = serde_json::to_string_pretty(index).map_err(|error| error.to_string())?;
    fs::write(profile_dir.join("index.json"), text).map_err(|error| error.to_string())
}

fn sync_index_with_files(profile_dir: &Path, index: &mut ModIndex) -> Result<(), String> {
    index.mods.retain(|mod_info| {
        profile_dir.join("mods").join(&mod_info.file_name).exists()
            || profile_dir
                .join("disabled-mods")
                .join(&mod_info.file_name)
                .exists()
    });
    sync_dir(profile_dir, index, "mods", true)?;
    sync_dir(profile_dir, index, "disabled-mods", false)
}

fn sync_dir(
    profile_dir: &Path,
    index: &mut ModIndex,
    dir_name: &str,
    enabled: bool,
) -> Result<(), String> {
    let dir = profile_dir.join(dir_name);
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(&dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_file()
            || !is_jar(&entry.path())
        {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if let Some(mod_info) = index
            .mods
            .iter_mut()
            .find(|mod_info| mod_info.file_name == file_name)
        {
            mod_info.enabled = enabled;
            continue;
        }
        index.mods.push(ModInfo {
            name: file_stem(&entry.path()).unwrap_or_else(|| file_name.clone()),
            project_id: String::new(),
            version_id: String::new(),
            file_name,
            sha512: String::new(),
            enabled,
            source: "local".to_string(),
            minecraft_version: String::new(),
            loader: String::new(),
        });
    }
    Ok(())
}

fn remove_if_exists(path: PathBuf) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn ensure_safe_file_name(file_name: &str) -> Result<String, String> {
    let invalid = file_name.is_empty()
        || file_name.contains("..")
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains(':')
        || !file_name.to_ascii_lowercase().ends_with(".jar");
    if invalid {
        Err("mod file name must be a .jar file inside the selected profile".to_string())
    } else {
        Ok(file_name.to_string())
    }
}

fn is_jar(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("jar"))
}

fn file_stem(path: &Path) -> Option<String> {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
}

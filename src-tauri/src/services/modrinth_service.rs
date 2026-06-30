use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    models::mod_info::{ModIndex, ModInfo},
    services::profile_service,
};

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

pub fn check_mod_updates(profile_path: String) -> Result<Vec<String>, String> {
    let profile_dir = profile_service::resolve_profile_path(&profile_path)?;
    let mut index = read_index(&profile_dir)?;
    sync_index_with_files(&profile_dir, &mut index)?;
    write_index(&profile_dir, &index)?;
    Ok(vec![])
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

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::models::{
    mod_info::ModIndex,
    profile::{Profile, ProfileSummary},
};

const DATA_DIR: &str = "taurine-data";

pub fn data_dir() -> Result<PathBuf, String> {
    std::env::current_dir()
        .map(|dir| dir.join(DATA_DIR))
        .map_err(|error| error.to_string())
}

pub fn ensure_base_dirs() -> Result<(), String> {
    let root = data_dir()?;
    for path in [
        root.join("profiles"),
        root.join("servers"),
        root.join("runtime/java"),
        root.join("runtime/minecraft"),
        root.join("runtime/loaders"),
        root.join("cache/downloads"),
        root.join("cache/modrinth"),
        root.join("cache/icons"),
        root.join("logs"),
    ] {
        fs::create_dir_all(path).map_err(|error| error.to_string())?;
    }
    let config = root.join("config.toml");
    if !config.exists() {
        fs::write(config, "data_version = 1\nlast_profile = \"\"\n")
            .map_err(|error| error.to_string())?;
    }
    let launcher_log = root.join("logs/launcher.log");
    if !launcher_log.exists() {
        fs::write(launcher_log, "").map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn create_profile(
    version: String,
    loader: String,
    name: String,
    loader_version: String,
    auto_memory: bool,
) -> Result<ProfileSummary, String> {
    ensure_safe_component(&version)?;
    ensure_safe_component(&loader)?;
    ensure_safe_component(&name)?;
    ensure_base_dirs()?;

    let profile = Profile::new(
        name.clone(),
        version.clone(),
        loader.clone(),
        loader_version,
        auto_memory,
    );
    let profile_dir = profile_dir(&version, &loader, &name)?;
    if profile_dir.exists() {
        return Err(
            "同じMinecraftバージョン、Loader、名前のプロファイルが既に存在します".to_string(),
        );
    }
    create_profile_dirs(&profile_dir)?;
    write_profile(&profile_dir.join("profile.toml"), &profile)?;
    write_index(&profile_dir.join("index.json"), &ModIndex::default())?;
    summarize_profile(&profile_dir)
}

pub fn list_profiles() -> Result<Vec<ProfileSummary>, String> {
    ensure_base_dirs()?;
    let profiles_root = data_dir()?.join("profiles");
    let mut summaries = Vec::new();
    if !profiles_root.exists() {
        return Ok(summaries);
    }
    for version in read_dirs(&profiles_root)? {
        for loader in read_dirs(&version)? {
            for profile_dir in read_dirs(&loader)? {
                if profile_dir.join("profile.toml").exists() {
                    summaries.push(summarize_profile(&profile_dir)?);
                }
            }
        }
    }
    summaries.sort_by(|a, b| {
        (&a.minecraft_version, &a.loader, &a.name).cmp(&(&b.minecraft_version, &b.loader, &b.name))
    });
    Ok(summaries)
}

pub fn read_profile(profile_path: String) -> Result<Profile, String> {
    let path = resolve_profile_path(&profile_path)?;
    let text = fs::read_to_string(path.join("profile.toml")).map_err(|error| error.to_string())?;
    toml::from_str(&text).map_err(|error| error.to_string())
}

pub fn update_profile(profile_path: String, profile: Profile) -> Result<ProfileSummary, String> {
    let path = resolve_profile_path(&profile_path)?;
    write_profile(&path.join("profile.toml"), &profile)?;
    summarize_profile(&path)
}

pub fn delete_profile(profile_path: String) -> Result<(), String> {
    let path = resolve_profile_path(&profile_path)?;
    fs::remove_dir_all(path).map_err(|error| error.to_string())
}

pub fn summarize_profile(profile_dir: &Path) -> Result<ProfileSummary, String> {
    let profile: Profile = toml::from_str(
        &fs::read_to_string(profile_dir.join("profile.toml")).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let enabled = count_jars(&profile_dir.join("mods"))?;
    let disabled = count_jars(&profile_dir.join("disabled-mods"))?;
    Ok(ProfileSummary {
        name: profile.name,
        minecraft_version: profile.minecraft_version,
        loader: profile.loader,
        loader_version: profile.loader_version,
        path: profile_dir.to_string_lossy().to_string(),
        mod_count: enabled + disabled,
        enabled_mod_count: enabled,
        disabled_mod_count: disabled,
        auto_memory: profile.launch.auto_memory,
        memory_max_mb: profile.launch.memory_max_mb,
        server_enabled: profile.server.enabled,
    })
}

pub fn resolve_profile_path(profile_path: &str) -> Result<PathBuf, String> {
    ensure_base_dirs()?;
    let root = data_dir()?
        .join("profiles")
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let path = PathBuf::from(profile_path);
    let canonical = path.canonicalize().map_err(|error| error.to_string())?;
    if !canonical.starts_with(&root) {
        return Err("プロファイルパスがtaurine-data/profilesの外を指しています".to_string());
    }
    Ok(canonical)
}

fn profile_dir(version: &str, loader: &str, name: &str) -> Result<PathBuf, String> {
    Ok(data_dir()?
        .join("profiles")
        .join(version)
        .join(loader)
        .join(name))
}

fn create_profile_dirs(profile_dir: &Path) -> Result<(), String> {
    for dir in [
        "mods",
        "disabled-mods",
        "config",
        "resourcepacks",
        "shaderpacks",
        "logs",
    ] {
        fs::create_dir_all(profile_dir.join(dir)).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn write_profile(path: &Path, profile: &Profile) -> Result<(), String> {
    let text = toml::to_string_pretty(profile).map_err(|error| error.to_string())?;
    fs::write(path, text).map_err(|error| error.to_string())
}

fn write_index(path: &Path, index: &ModIndex) -> Result<(), String> {
    let text = serde_json::to_string_pretty(index).map_err(|error| error.to_string())?;
    fs::write(path, text).map_err(|error| error.to_string())
}

fn read_dirs(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            entries.push(entry.path());
        }
    }
    Ok(entries)
}

fn count_jars(path: &Path) -> Result<usize, String> {
    if !path.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
        {
            count += 1;
        }
    }
    Ok(count)
}

fn ensure_safe_component(value: &str) -> Result<(), String> {
    let invalid = value.is_empty()
        || value.contains("..")
        || value.contains('/')
        || value.contains('\\')
        || value.contains(':');
    if invalid {
        Err("名前、バージョン、Loaderにはパス記号を含められません".to_string())
    } else {
        Ok(())
    }
}

import { invoke } from "@tauri-apps/api/core";

export interface ProfileSummary {
  name: string;
  minecraft_version: string;
  loader: string;
  loader_version: string;
  path: string;
  mod_count: number;
  enabled_mod_count: number;
  disabled_mod_count: number;
  auto_memory: boolean;
  memory_max_mb: number;
  server_enabled: boolean;
}

export interface MemoryPlan {
  total_memory_mb: number;
  os_reserved_mb: number;
  mod_count: number;
  recommended_mb: number;
  capped_by_system: boolean;
  manual_override: boolean;
}

export interface CreateProfileInput {
  version: string;
  loader: string;
  name: string;
  loaderVersion: string;
  autoMemory: boolean;
}


export interface MicrosoftDeviceCode {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
  message: string;
}

export interface MicrosoftTokenResult {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  scope: string;
}export interface JavaDetection {
  found: boolean;
  path: string;
  version: string;
}

export interface LaunchResult {
  command_preview: string;
  log_path: string;
}

export interface ModInfo {
  name: string;
  project_id: string;
  version_id: string;
  file_name: string;
  sha512: string;
  enabled: boolean;
  source: string;
  minecraft_version: string;
  loader: string;
}

export interface ModrinthSearchResult {
  project_id: string;
  title: string;
  description: string;
  icon_url: string;
  downloads: number;
}

export interface ModUpdateResult {
  name: string;
  file_name: string;
  current_version_id: string;
  latest_version_id: string;
  latest_version_number: string;
  latest_file_name: string;
}

export interface ModrinthVersionResult {
  version_id: string;
  name: string;
  version_number: string;
  file_name: string;
  download_url: string;
}

export function createProfile(input: CreateProfileInput) {
  return invoke<ProfileSummary>("create_profile", {
    version: input.version,
    loader: input.loader,
    name: input.name,
    loaderVersion: input.loaderVersion,
    autoMemory: input.autoMemory,
  });
}

export function listProfiles() {
  return invoke<ProfileSummary[]>("list_profiles");
}

export function calculateMemory(profilePath: string) {
  return invoke<MemoryPlan>("calculate_memory", { profilePath });
}


export function beginMicrosoftDeviceLogin() {
  return invoke<MicrosoftDeviceCode>("begin_microsoft_device_login");
}

export function pollMicrosoftDeviceLogin(deviceCode: string) {
  return invoke<MicrosoftTokenResult | null>("poll_microsoft_device_login", { deviceCode });
}export function detectJava() {
  return invoke<JavaDetection>("detect_java");
}

export function launchMinecraft(profilePath: string) {
  return invoke<LaunchResult>("launch_minecraft", { profilePath });
}

export function listInstalledMods(profilePath: string) {
  return invoke<ModInfo[]>("list_installed_mods", { profilePath });
}

export function searchModrinth(query: string, version: string, loader: string) {
  return invoke<ModrinthSearchResult[]>("search_modrinth", { query, version, loader });
}

export function getModrinthVersions(projectId: string, version: string, loader: string) {
  return invoke<ModrinthVersionResult[]>("get_modrinth_versions", { projectId, version, loader });
}

export function installMod(profilePath: string, projectId: string, versionId: string) {
  return invoke<ModInfo>("install_mod", { profilePath, projectId, versionId });
}

export function checkModUpdates(profilePath: string) {
  return invoke<ModUpdateResult[]>("check_mod_updates", { profilePath });
}

export function updateMod(profilePath: string, fileName: string, versionId: string) {
  return invoke<ModInfo>("update_mod", { profilePath, fileName, versionId });
}

export function enableMod(profilePath: string, fileName: string) {
  return invoke<void>("enable_mod", { profilePath, fileName });
}

export function disableMod(profilePath: string, fileName: string) {
  return invoke<void>("disable_mod", { profilePath, fileName });
}

export function removeMod(profilePath: string, fileName: string) {
  return invoke<void>("remove_mod", { profilePath, fileName });
}
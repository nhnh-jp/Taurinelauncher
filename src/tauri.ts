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

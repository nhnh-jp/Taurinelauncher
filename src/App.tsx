import { useEffect, useMemo, useState } from "react";
import { Boxes, FileText, Home, ListPlus, LucideIcon, Play, RefreshCw, Search, Server, Settings, Trash2 } from "lucide-react";
import { calculateMemory, checkModUpdates, createProfile, disableMod, enableMod, getModrinthVersions, installMod, listInstalledMods, listProfiles, MemoryPlan, ModInfo, ModrinthSearchResult, ModUpdateResult, ProfileSummary, removeMod, searchModrinth } from "./tauri";

type Page = "home" | "profiles" | "create" | "mods" | "servers" | "settings" | "logs";

const navItems: Array<{ id: Page; label: string; icon: LucideIcon }> = [
  { id: "home", label: "Home", icon: Home },
  { id: "profiles", label: "Profiles", icon: Boxes },
  { id: "create", label: "Create", icon: ListPlus },
  { id: "mods", label: "Mods", icon: Search },
  { id: "servers", label: "Servers", icon: Server },
  { id: "settings", label: "Settings", icon: Settings },
  { id: "logs", label: "Logs", icon: FileText },
];

export function App() {
  const [page, setPage] = useState<Page>("home");
  const [profiles, setProfiles] = useState<ProfileSummary[]>([]);
  const [selectedPath, setSelectedPath] = useState("");
  const [status, setStatus] = useState("Ready");
  const [loading, setLoading] = useState(false);

  const selectedProfile = useMemo(
    () => profiles.find((profile) => profile.path === selectedPath) ?? profiles[0],
    [profiles, selectedPath],
  );

  async function refreshProfiles() {
    setLoading(true);
    try {
      const nextProfiles = await listProfiles();
      setProfiles(nextProfiles);
      if (!selectedPath && nextProfiles[0]) setSelectedPath(nextProfiles[0].path);
      setStatus(`Loaded ${nextProfiles.length} profiles`);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refreshProfiles();
  }, []);

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand"><span className="brand-mark">T</span><div><strong>Taurine</strong><span>Launcher</span></div></div>
        <nav className="nav-list">
          {navItems.map((item) => {
            const Icon = item.icon;
            return <button key={item.id} className={page === item.id ? "nav-item active" : "nav-item"} onClick={() => setPage(item.id)} type="button"><Icon size={18} />{item.label}</button>;
          })}
        </nav>
      </aside>
      <main className="main-area">
        <header className="topbar"><div><h1>{pageTitle(page)}</h1><p>{status}</p></div><button className="icon-button" onClick={refreshProfiles} disabled={loading} title="Refresh" type="button"><RefreshCw size={18} /></button></header>
        {page === "home" && <HomePage profile={selectedProfile} onCreate={() => setPage("create")} />}
        {page === "profiles" && <ProfilesPage profiles={profiles} selectedPath={selectedPath} onSelect={setSelectedPath} onCreate={() => setPage("create")} />}
        {page === "create" && <CreateProfilePage onCreated={refreshProfiles} setStatus={setStatus} />}
        {page === "mods" && <ModsPage profiles={profiles} selectedPath={selectedPath} onSelect={setSelectedPath} />}
        {page === "servers" && <ServersPage />}
        {page === "settings" && <SettingsPage />}
        {page === "logs" && <LogsPage profile={selectedProfile} />}
      </main>
    </div>
  );
}

function pageTitle(page: Page) {
  return { home: "Home", profiles: "Profiles", create: "Create Profile", mods: "Mods", servers: "Servers", settings: "Settings", logs: "Logs" }[page];
}

function HomePage({ profile, onCreate }: { profile?: ProfileSummary; onCreate: () => void }) {
  return <section className="content-grid"><div className="panel primary-panel"><span className="eyebrow">Last Profile</span><h2>{profile ? profile.name : "No profile"}</h2><p>{profile ? `${profile.minecraft_version} / ${profile.loader} / ${profile.mod_count} mods` : "Create a profile to start."}</p><div className="actions"><button className="primary-button" type="button" disabled={!profile}><Play size={18} />Launch</button><button className="secondary-button" onClick={onCreate} type="button"><ListPlus size={18} />Create</button></div></div><div className="panel"><h3>Status</h3><dl className="status-list"><div><dt>Layout</dt><dd>Minecraft Version / Loader / Profile</dd></div><div><dt>Phase</dt><dd>2: local mod file management</dd></div></dl></div></section>;
}

function ProfilesPage({ profiles, selectedPath, onSelect, onCreate }: { profiles: ProfileSummary[]; selectedPath: string; onSelect: (path: string) => void; onCreate: () => void }) {
  return <section className="panel"><div className="section-header"><h2>Profiles</h2><button className="secondary-button" onClick={onCreate} type="button"><ListPlus size={17} />New</button></div><div className="table"><div className="table-row table-head"><span>Name</span><span>Version</span><span>Loader</span><span>Mods</span><span>Memory</span><span /></div>{profiles.map((profile) => <button className={profile.path === selectedPath ? "table-row selected" : "table-row"} key={profile.path} onClick={() => onSelect(profile.path)} type="button"><span>{profile.name}</span><span>{profile.minecraft_version}</span><span>{profile.loader}</span><span>{profile.enabled_mod_count}/{profile.mod_count}</span><span>{profile.auto_memory ? "Auto" : `${profile.memory_max_mb}MB`}</span><span className="row-actions"><Trash2 size={16} /></span></button>)}</div></section>;
}

function CreateProfilePage({ onCreated, setStatus }: { onCreated: () => Promise<void>; setStatus: (status: string) => void }) {
  const [name, setName] = useState("main");
  const [version, setVersion] = useState("1.21.1");
  const [loader, setLoader] = useState("fabric");
  const [loaderVersion, setLoaderVersion] = useState("latest");
  const [autoMemory, setAutoMemory] = useState(true);
  async function submit(event: React.FormEvent) { event.preventDefault(); try { const profile = await createProfile({ name, version, loader, loaderVersion, autoMemory }); setStatus(`Created ${profile.name}`); await onCreated(); } catch (error) { setStatus(error instanceof Error ? error.message : String(error)); } }
  return <form className="panel form-panel" onSubmit={submit}><label>Profile name<input value={name} onChange={(event) => setName(event.target.value)} required /></label><label>Minecraft version<input value={version} onChange={(event) => setVersion(event.target.value)} required /></label><label>Loader<select value={loader} onChange={(event) => setLoader(event.target.value)}><option value="fabric">Fabric</option><option value="forge">Forge</option><option value="neoforge">NeoForge</option><option value="quilt">Quilt</option><option value="vanilla">Vanilla</option></select></label><label>Loader version<input value={loaderVersion} onChange={(event) => setLoaderVersion(event.target.value)} /></label><label className="checkbox-row"><input checked={autoMemory} onChange={(event) => setAutoMemory(event.target.checked)} type="checkbox" />Auto memory</label><button className="primary-button" type="submit"><ListPlus size={18} />Create</button></form>;
}

function ModsPage({ profiles, selectedPath, onSelect }: { profiles: ProfileSummary[]; selectedPath: string; onSelect: (path: string) => void }) {
  const [mods, setMods] = useState<ModInfo[]>([]);
  const [results, setResults] = useState<ModrinthSearchResult[]>([]);
  const [updates, setUpdates] = useState<ModUpdateResult[]>([]);
  const [query, setQuery] = useState("");
  const [busyFile, setBusyFile] = useState("");
  const [busyProject, setBusyProject] = useState("");
  const [message, setMessage] = useState("");
  const selected = profiles.find((profile) => profile.path === selectedPath);

  async function refreshMods() {
    if (!selectedPath) {
      setMods([]);
      return;
    }
    try {
      const nextMods = await listInstalledMods(selectedPath);
      setMods(nextMods);
      setMessage(`Loaded ${nextMods.length} mods`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
  }

  async function submitSearch(event: React.FormEvent) {
    event.preventDefault();
    if (!selected || !query.trim()) return;
    setBusyProject("search");
    try {
      const nextResults = await searchModrinth(query, selected.minecraft_version, selected.loader);
      setResults(nextResults);
      setMessage(`Found ${nextResults.length} Modrinth projects`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusyProject("");
    }
  }

  async function installLatest(result: ModrinthSearchResult) {
    if (!selected) return;
    setBusyProject(result.project_id);
    try {
      const versions = await getModrinthVersions(result.project_id, selected.minecraft_version, selected.loader);
      const latest = versions[0];
      if (!latest) {
        setMessage("No compatible version was found for this profile");
        return;
      }
      const installed = await installMod(selected.path, result.project_id, latest.version_id);
      setMessage(`Installed ${installed.file_name}`);
      await refreshMods();
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusyProject("");
    }
  }
  async function checkUpdates() {
    if (!selected) return;
    setBusyProject("updates");
    try {
      const nextUpdates = await checkModUpdates(selected.path);
      setUpdates(nextUpdates);
      setMessage(nextUpdates.length === 0 ? "All Modrinth mods are up to date" : `Found ${nextUpdates.length} mod updates`);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusyProject("");
    }
  }

  async function runModAction(fileName: string, action: () => Promise<void>) {
    setBusyFile(fileName);
    try {
      await action();
      await refreshMods();
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusyFile("");
    }
  }

  useEffect(() => {
    void refreshMods();
  }, [selectedPath]);

  return <section className="panel"><div className="section-header"><h2>Mods</h2><select value={selectedPath} onChange={(event) => onSelect(event.target.value)}>{profiles.map((profile) => <option key={profile.path} value={profile.path}>{profile.minecraft_version}/{profile.loader}/{profile.name}</option>)}</select></div><div className="mod-toolbar"><form className="search-bar" onSubmit={submitSearch}><input placeholder="Search Modrinth" value={query} onChange={(event) => setQuery(event.target.value)} disabled={!selected || busyProject === "search"} /><button className="secondary-button" disabled={!selected || !query.trim() || busyProject === "search"} type="submit"><Search size={17} />Search</button></form><button className="secondary-button" disabled={!selected || busyProject === "updates"} onClick={checkUpdates} type="button"><RefreshCw size={17} />Check updates</button></div><p className="muted">{selected ? `Current mods: ${selected.mod_count}. Enabled ${selected.enabled_mod_count} / Disabled ${selected.disabled_mod_count}` : "Create a profile to manage mods."}</p><p className="muted">{message}</p>{updates.length > 0 && <div className="update-results">{updates.map((update) => <div className="update-row" key={update.file_name}><div><strong>{update.name}</strong><span>{update.file_name} to {update.latest_file_name}</span></div><span>{update.latest_version_number}</span></div>)}</div>}{results.length > 0 && <div className="search-results">{results.map((result) => <div className="result-row" key={result.project_id}>{result.icon_url ? <img alt="" src={result.icon_url} /> : <span className="result-icon">M</span>}<div><strong>{result.title}</strong><span>{result.description}</span></div><span className="downloads">{result.downloads.toLocaleString()} dl</span><button className="secondary-button" disabled={!selected || busyProject === result.project_id} onClick={() => installLatest(result)} type="button">Install</button></div>)}</div>}<div className="mod-list">{mods.length === 0 ? <p className="empty-state">Put .jar files in mods/ or disabled-mods/, or install from Modrinth search.</p> : mods.map((mod) => <div className="mod-row" key={mod.file_name}><div><strong>{mod.name}</strong><span>{mod.file_name}</span></div><span className={mod.enabled ? "state enabled" : "state disabled"}>{mod.enabled ? "Enabled" : "Disabled"}</span><button className="secondary-button" disabled={busyFile === mod.file_name} onClick={() => runModAction(mod.file_name, () => mod.enabled ? disableMod(selectedPath, mod.file_name) : enableMod(selectedPath, mod.file_name))} type="button">{mod.enabled ? "Disable" : "Enable"}</button><button className="icon-button" disabled={busyFile === mod.file_name} onClick={() => runModAction(mod.file_name, () => removeMod(selectedPath, mod.file_name))} title="Remove" type="button"><Trash2 size={16} /></button></div>)}</div></section>;
}

function ServersPage() { return <section className="panel"><h2>Server profiles</h2><p className="muted">servers/*.toml loading and required mod installation are planned for Phase 3.</p></section>; }
function SettingsPage() { return <section className="panel settings-grid"><label>Java path<input value="auto" readOnly /></label><label>Data directory<input value="taurine-data/" readOnly /></label><label>Theme<select defaultValue="system"><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></label></section>; }
function LogsPage({ profile }: { profile?: ProfileSummary }) { const [memory, setMemory] = useState<MemoryPlan | null>(null); useEffect(() => { if (!profile) { setMemory(null); return; } void calculateMemory(profile.path).then(setMemory); }, [profile]); return <section className="panel log-panel"><h2>Logs</h2><pre>{profile ? `profile: ${profile.minecraft_version}/${profile.loader}/${profile.name}\nrecommended_memory: ${memory ? `${memory.recommended_mb}MB` : "calculating"}\nlauncher.log connection is planned for Phase 4.` : "No profile."}</pre></section>; }
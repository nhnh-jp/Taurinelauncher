import { useEffect, useMemo, useState } from "react";
import { Boxes, FileText, Home, ListPlus, LucideIcon, Play, RefreshCw, Search, Server, Settings, Trash2 } from "lucide-react";
import { calculateMemory, createProfile, listProfiles, MemoryPlan, ProfileSummary } from "./tauri";

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
  const [status, setStatus] = useState("準備完了");
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
      setStatus(`${nextProfiles.length}件のプロファイルを読み込みました`);
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
        <header className="topbar"><div><h1>{pageTitle(page)}</h1><p>{status}</p></div><button className="icon-button" onClick={refreshProfiles} disabled={loading} title="更新" type="button"><RefreshCw size={18} /></button></header>
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
  return <section className="content-grid"><div className="panel primary-panel"><span className="eyebrow">Last Profile</span><h2>{profile ? profile.name : "プロファイル未作成"}</h2><p>{profile ? `${profile.minecraft_version} / ${profile.loader} / MOD ${profile.mod_count}件` : "最初のプロファイルを作成してください。"}</p><div className="actions"><button className="primary-button" type="button" disabled={!profile}><Play size={18} />起動</button><button className="secondary-button" onClick={onCreate} type="button"><ListPlus size={18} />作成</button></div></div><div className="panel"><h3>状態</h3><dl className="status-list"><div><dt>管理単位</dt><dd>Minecraft Version / Loader / Profile</dd></div><div><dt>Phase</dt><dd>1: profile.toml と自動メモリ計算</dd></div></dl></div></section>;
}

function ProfilesPage({ profiles, selectedPath, onSelect, onCreate }: { profiles: ProfileSummary[]; selectedPath: string; onSelect: (path: string) => void; onCreate: () => void }) {
  return <section className="panel"><div className="section-header"><h2>プロファイル一覧</h2><button className="secondary-button" onClick={onCreate} type="button"><ListPlus size={17} />新規作成</button></div><div className="table"><div className="table-row table-head"><span>名前</span><span>Version</span><span>Loader</span><span>MOD</span><span>Memory</span><span /></div>{profiles.map((profile) => <button className={profile.path === selectedPath ? "table-row selected" : "table-row"} key={profile.path} onClick={() => onSelect(profile.path)} type="button"><span>{profile.name}</span><span>{profile.minecraft_version}</span><span>{profile.loader}</span><span>{profile.enabled_mod_count}/{profile.mod_count}</span><span>{profile.auto_memory ? "Auto" : `${profile.memory_max_mb}MB`}</span><span className="row-actions"><Trash2 size={16} /></span></button>)}</div></section>;
}

function CreateProfilePage({ onCreated, setStatus }: { onCreated: () => Promise<void>; setStatus: (status: string) => void }) {
  const [name, setName] = useState("main");
  const [version, setVersion] = useState("1.21.1");
  const [loader, setLoader] = useState("fabric");
  const [loaderVersion, setLoaderVersion] = useState("latest");
  const [autoMemory, setAutoMemory] = useState(true);
  async function submit(event: React.FormEvent) { event.preventDefault(); try { const profile = await createProfile({ name, version, loader, loaderVersion, autoMemory }); setStatus(`${profile.name} を作成しました`); await onCreated(); } catch (error) { setStatus(error instanceof Error ? error.message : String(error)); } }
  return <form className="panel form-panel" onSubmit={submit}><label>プロファイル名<input value={name} onChange={(event) => setName(event.target.value)} required /></label><label>Minecraftバージョン<input value={version} onChange={(event) => setVersion(event.target.value)} required /></label><label>Loader<select value={loader} onChange={(event) => setLoader(event.target.value)}><option value="fabric">Fabric</option><option value="forge">Forge</option><option value="neoforge">NeoForge</option><option value="quilt">Quilt</option><option value="vanilla">Vanilla</option></select></label><label>Loaderバージョン<input value={loaderVersion} onChange={(event) => setLoaderVersion(event.target.value)} /></label><label className="checkbox-row"><input checked={autoMemory} onChange={(event) => setAutoMemory(event.target.checked)} type="checkbox" />自動メモリ設定</label><button className="primary-button" type="submit"><ListPlus size={18} />作成</button></form>;
}

function ModsPage({ profiles, selectedPath, onSelect }: { profiles: ProfileSummary[]; selectedPath: string; onSelect: (path: string) => void }) {
  const selected = profiles.find((profile) => profile.path === selectedPath);
  return <section className="panel"><div className="section-header"><h2>MOD管理</h2><select value={selectedPath} onChange={(event) => onSelect(event.target.value)}>{profiles.map((profile) => <option key={profile.path} value={profile.path}>{profile.minecraft_version}/{profile.loader}/{profile.name}</option>)}</select></div><div className="search-bar"><input placeholder="Modrinth検索はPhase 2で実装" disabled /><button className="secondary-button" disabled type="button"><Search size={17} />検索</button></div><p className="muted">{selected ? `現在のMOD数: ${selected.mod_count}件。有効 ${selected.enabled_mod_count}件 / 無効 ${selected.disabled_mod_count}件` : "プロファイルを作成するとMOD管理を開始できます。"}</p></section>;
}

function ServersPage() { return <section className="panel"><h2>サーバープロファイル</h2><p className="muted">servers/*.toml の読み込みと必須MOD導入はPhase 3で実装します。</p></section>; }
function SettingsPage() { return <section className="panel settings-grid"><label>Javaパス<input value="auto" readOnly /></label><label>データ保存場所<input value="taurine-data/" readOnly /></label><label>テーマ<select defaultValue="system"><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></label></section>; }
function LogsPage({ profile }: { profile?: ProfileSummary }) { const [memory, setMemory] = useState<MemoryPlan | null>(null); useEffect(() => { if (!profile) { setMemory(null); return; } void calculateMemory(profile.path).then(setMemory); }, [profile]); return <section className="panel log-panel"><h2>ログ</h2><pre>{profile ? `profile: ${profile.minecraft_version}/${profile.loader}/${profile.name}\nrecommended_memory: ${memory ? `${memory.recommended_mb}MB` : "calculating"}\nlauncher.log はPhase 4で接続します。` : "プロファイルがありません。"}</pre></section>; }


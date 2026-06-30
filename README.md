# Taurine Launcher

Taurine Launcher は、Minecraft Java Edition のMOD環境を「Minecraftバージョン / Loader / Profile」単位で分離して管理する軽量ランチャーです。

重い統合ランチャーではなく、起動の速さ、壊れにくいファイル構成、手動復旧しやすい設計、サーバー参加用プロファイルの作りやすさを優先します。

## コンセプト

管理単位は必ず次の階層にします。

```txt
Minecraftバージョン
  └ Loader
      └ Profile
```

例:

```txt
profiles/1.21.1/fabric/main/
profiles/1.21.1/fabric/neha-server/
profiles/1.20.1/forge/test/
```

1プロファイルを1つの game directory として扱い、`mods`、`disabled-mods`、`config`、`resourcepacks`、`shaderpacks`、`logs` をプロファイルごとに分離します。

## 主な機能

- プロファイル作成
- プロファイル一覧表示
- Minecraftバージョン別管理
- Loader別管理
- profile.toml の保存 / 読み込み
- プロファイルごとのMOD個別管理
- Modrinth API検索とダウンロード
- MODの有効化 / 無効化
- 自動メモリ調整
- Java検出
- Minecraft起動処理の土台
- 起動ログ表示
- サーバー用プロファイル作成

Phase 1では、Tauri v2 + React + TypeScript の土台、基本ディレクトリ作成、プロファイル作成、一覧表示、`profile.toml` の保存 / 読み込み、自動メモリ計算を実装しています。

## 作らない機能

初期版では以下は作りません。

- VC機能
- SNS機能
- チャット機能
- 独自MOD投稿サイト
- CurseForge完全互換
- 大規模Modpack管理
- アカウント高度管理

## ファイル構成

ランチャーデータはアプリの実行ディレクトリ直下の `taurine-data/` に作成します。

```txt
taurine-data/
  config.toml

  profiles/
    1.21.1/
      fabric/
        main/
          profile.toml
          index.json
          mods/
          disabled-mods/
          config/
          resourcepacks/
          shaderpacks/
          logs/

  servers/
    neha-server.toml

  runtime/
    java/
    minecraft/
    loaders/

  cache/
    downloads/
    modrinth/
    icons/

  logs/
    launcher.log
```

## プロファイル管理

`profile.toml` は次の形式です。

```toml
name = "main"
minecraft_version = "1.21.1"
loader = "fabric"
loader_version = "latest"

[launch]
auto_memory = true
memory_min_mb = 512
memory_max_mb = 4096
java_path = "auto"
extra_jvm_args = []
extra_game_args = []

[game]
resolution_width = 1280
resolution_height = 720
fullscreen = false

[mods]
check_updates_on_start = true
auto_install_dependencies = true

[server]
enabled = false
name = ""
address = ""
port = 25565
```

MODは `index.json` で管理します。ただし、復旧しやすさを優先し、`index.json` だけに依存せず `mods/` と `disabled-mods/` の実ファイル状態も見ます。

## サーバーランチャー機能

サーバー設定は `servers/` に TOML で置きます。Phase 3で、サーバー設定の読み込み、必須MOD確認、Modrinthからの導入、`profiles/{minecraft_version}/{loader}/{server_name}/` の生成を実装します。

## 自動メモリ調整

搭載メモリとMOD数から Xmx を決定します。

```txt
MOD数 0〜20     -> 2048MB
MOD数 21〜80    -> 4096MB
MOD数 81〜150   -> 6144MB
MOD数 151以上   -> 8192MB
```

ただしOS用に最低2GBを残します。ユーザーが手動指定した場合は手動設定を優先します。

## 開発環境

- Rust
- Node.js
- npm
- Tauri v2 prerequisites

依存関係のインストール:

```bash
npm install
```

## 起動方法

```bash
npm run tauri dev
```

フロントエンドだけ確認する場合:

```bash
npm run dev
```

## ビルド方法

```bash
npm run tauri build
```

## ロードマップ

### Phase 1

- Tauri v2 + React + TypeScriptの初期構成
- README.md作成
- 基本ディレクトリ作成
- プロファイル作成
- プロファイル一覧表示
- profile.tomlの保存 / 読み込み
- 自動メモリ計算

### Phase 2

- Modrinth検索
- MODダウンロード
- index.json作成
- MOD有効化 / 無効化

### Phase 3

- サーバー設定ファイル読み込み
- サーバー用プロファイル作成
- 必須MODの自動導入

### Phase 4

- Java検出
- Minecraft起動処理
- ログ表示

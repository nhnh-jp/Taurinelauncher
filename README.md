# Taurinelauncher
rust製のlauncher
# Taurine Launcher

Taurine Launcher は、Tauri で作る軽量な Minecraft Java Edition 用ランチャーです。

目的は、MOD環境をわかりやすく分けて管理し、できるだけ軽く・速く Minecraft を起動できるようにすることです。

## コンセプト

Taurine Launcher は、Minecraft の環境を以下の単位で管理します。

```txt
Minecraftバージョン
  └ Loader
      └ Profile
```

例：

```txt
1.21.1
  └ fabric
      ├ main
      ├ fps
      └ neha-server

1.20.1
  └ forge
      └ modpack-test
```

各プロファイルは、それぞれ独立した `mods`、`config`、`resourcepacks`、`shaderpacks`、起動設定を持ちます。

これにより、違うバージョンや違うLoaderのMODが混ざる事故を防ぎます。

## 主な機能

予定している機能は以下です。

* 軽量なTauri製ランチャー
* Minecraftプロファイル管理
* Minecraftバージョン別管理
* Fabric / Forge / NeoForge 対応
* プロファイルごとのMOD個別管理
* Modrinth APIを使ったMOD検索
* ModrinthからのMODダウンロード
* MODの有効化 / 無効化
* MOD更新確認
* 自動メモリ調整
* Java検出
* 起動ログ表示
* サーバー用プロファイル作成機能

## 作らない機能

初期版では以下の機能は作りません。

* VC機能
* SNS機能
* チャット機能
* 独自MOD投稿サイト
* 大規模なModpack管理機能
* CurseForge完全互換

Taurine Launcher は多機能すぎるランチャーではなく、軽くて管理しやすいランチャーを目指します。

## サーバーランチャー機能

サーバーランチャー機能では、特定のMinecraftサーバーに参加するための環境を簡単に作成できます。

例：

```txt
neha-server
  Minecraft: 1.21.1
  Loader: Fabric
  必須MOD:
    - Sodium
    - Iris
    - Lithium
```

ユーザーはサーバーを選ぶだけで、必要なプロファイルが自動生成されます。

予定している処理：

```txt
サーバーを選択
  ↓
サーバー設定を読み込み
  ↓
Minecraftバージョンを確認
  ↓
Loaderを確認
  ↓
必要MODを確認
  ↓
ModrinthからMODをダウンロード
  ↓
専用プロファイルを作成
  ↓
Minecraftを起動
```

サーバーごとにプロファイルを分けることで、別サーバーのMOD構成と混ざらないようにします。

## ファイル構成

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

        neha-server/
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

## プロファイル設定

`profile.toml` の例です。

```toml
name = "neha-server"
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
enabled = true
name = "neha-server"
address = "play.example.com"
port = 25565
```

## サーバー設定

`servers/neha-server.toml` の例です。

```toml
name = "neha-server"
description = "neha鯖用の公式プロファイル"
minecraft_version = "1.21.1"
loader = "fabric"
loader_version = "latest"
address = "play.example.com"
port = 25565

[[mods]]
name = "Sodium"
source = "modrinth"
project_id = "AANobbMI"
required = true

[[mods]]
name = "Iris"
source = "modrinth"
project_id = "YL57xq9U"
required = false
```

## MOD管理

各プロファイルには `index.json` を置き、インストール済みMODを管理します。

```json
{
  "schema_version": 1,
  "mods": [
    {
      "name": "Sodium",
      "project_id": "AANobbMI",
      "version_id": "example",
      "file_name": "sodium-fabric.jar",
      "sha512": "example",
      "enabled": true,
      "source": "modrinth",
      "minecraft_version": "1.21.1",
      "loader": "fabric"
    }
  ]
}
```

MODの有効化 / 無効化は、ファイル移動で行います。

```txt
mods/
  sodium.jar

disabled-mods/
  old-mod.jar
```

`mods` にあるMODは有効、`disabled-mods` にあるMODは無効です。

## 自動メモリ調整

搭載メモリとMOD数から、Minecraftに割り当てるメモリを自動で決めます。

目安：

```txt
MOD数 0〜20     : 2048MB
MOD数 21〜80    : 4096MB
MOD数 81〜150   : 6144MB
MOD数 151以上   : 8192MB
```

ただし、OS用のメモリは必ず残します。
ユーザーが手動でメモリを指定した場合は、手動設定を優先します。

## 開発環境

必要なもの：

* Rust
* Node.js
* Tauri v2
* npm / pnpm / yarn

インストール：

```bash
npm install
```

開発起動：

```bash
npm run tauri dev
```

ビルド：

```bash
npm run tauri build
```

## ロードマップ

### Phase 1

* プロファイル作成
* プロファイル一覧表示
* ファイル構成の自動生成
* `profile.toml` の保存 / 読み込み
* 自動メモリ計算

### Phase 2

* Modrinth検索
* MODダウンロード
* `index.json` 作成
* MOD有効化 / 無効化

### Phase 3

* Java検出
* Minecraft起動
* ログ表示

### Phase 4

* サーバー用プロファイル作成
* サーバー設定ファイル読み込み
* 必須MODの自動導入
* サーバー別プロファイルの管理

### Phase 5

* MOD更新確認
* 依存関係の自動導入
* UI改善

## 方針

Taurine Launcher は、巨大な多機能ランチャーではなく、軽くて管理しやすいランチャーを目指します。

重視すること：

* 軽い
* 起動が速い
* MODが混ざらない
* ファイル構成がわかりやすい
* 壊れても手動で直しやすい
* サーバー参加用の環境を簡単に作れる

# HANDOFF

## 現在の状態

Phase 1 の実装を進めました。

実装済み:

- Tauri v2 + React + TypeScript の初期構成
- 日本語 README.md の作成
- フロントエンド画面の土台
  - Home
  - Profiles
  - CreateProfile
  - Mods
  - Servers
  - Settings
  - Logs
- Rust 側の責務分割
  - commands/
  - services/
  - models/
- プロファイル作成
- プロファイル一覧表示
- profile.toml の保存 / 読み込み / 更新 / 削除 command
- taurine-data/ の基本ディレクトリ作成
- index.json の初期生成
- mods/ と disabled-mods/ の jar 数集計
- 自動メモリ計算
- Phase 2 以降の command 名だけは用意し、未実装エラーを返す状態

## 重要なファイル

- `package.json`
- `src/App.tsx`
- `src/tauri.ts`
- `src/styles.css`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src-tauri/src/main.rs`
- `src-tauri/src/services/profile_service.rs`
- `src-tauri/src/services/memory_service.rs`
- `src-tauri/src/models/profile.rs`
- `README.md`

## 検証結果

通ったもの:

```bash
npm.cmd install
npm.cmd run build
cargo check
```

`cargo check` は成功していますが、以下の warning が1件あります。

```txt
function `ensure_data_dirs` is never used
```

`src-tauri/src/commands/fs.rs` の `ensure_data_dirs` は command として invoke_handler に登録していないためです。不要なら削除、使うなら `main.rs` の `generate_handler!` に追加してください。

## 注意点

- `apply_patch` は Windows sandbox helper 欠落で失敗しました。
- そのためファイル作成は PowerShell で実施しています。
- PowerShell の `Set-Content -Encoding UTF8` は BOM を付けるため、JSONファイルは BOM なしで書き直しました。
- npm は PowerShell の実行ポリシーで `npm` が失敗したため、`npm.cmd` を使用しました。
- `npm install` 後に audit 警告が出ています。
  - 2 vulnerabilities: 1 moderate, 1 high
  - まだ修正していません。

## 次にやるなら

1. `ensure_data_dirs` warning の処理
2. `npm audit` の確認
3. 実際に `npm run tauri dev` でUIからプロファイル作成を動作確認
4. Phase 2: Modrinth検索、MODダウンロード、index.json反映、enable/disable実装

## 作業中の git 状態

新規追加・変更が多数あります。まだ commit はしていません。
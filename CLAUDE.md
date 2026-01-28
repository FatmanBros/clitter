# Clitter

クリップボード管理システムのデスクトップアプリケーション

## 技術スタック

- **フロントエンド**: Svelte 5 + TypeScript + Tailwind CSS + Vite
- **バックエンド**: Tauri 2 (Rust)
- **データベース**: SQLite (sqlx)
- **暗号化**: AES-GCM, Argon2

## プロジェクト構造

```
clitter/
├── src/              # フロントエンド (Svelte)
│   ├── App.svelte
│   ├── lib/
│   └── styles/
├── src-tauri/        # バックエンド (Rust)
│   ├── src/
│   └── tauri.conf.json
└── package.json
```

## 開発コマンド

```bash
cd clitter

# 開発サーバー起動
npm run tauri:dev

# ビルド
npm run tauri:build

# フロントエンドのみ
npm run dev
```

## Windows exe 生成

### 前提条件

1. **Rust**: https://rustup.rs/ または `winget install Rustlang.Rustup`
2. **Node.js**: https://nodejs.org/ (LTS版)
3. **Visual Studio Build Tools**: 「C++ によるデスクトップ開発」ワークロード

### ビルド

```powershell
cd clitter
npm install
npm run tauri build
```

### 出力ファイル

```
src-tauri/target/release/
├── clitter.exe                      # 単体実行ファイル

src-tauri/target/release/bundle/
├── msi/Clitter_0.1.0_x64_en-US.msi  # MSI インストーラー
└── nsis/Clitter_0.1.0_x64-setup.exe # NSIS インストーラー（推奨）
```

## GitHub Actions 自動リリース

コード管理リポジトリにpushすると、公開リポジトリに自動でリリースを作成。

### セットアップ

1. 公開用リポジトリ `clitter-releases` を作成
2. Personal Access Token (PAT) を作成（`repo`スコープ）
3. コード管理リポジトリのSecretに `RELEASE_TOKEN` として登録

### 実行

```bash
git tag v0.1.0
git push origin v0.1.0
```

### 生成される成果物

| OS | ファイル |
|----|----------|
| Windows | `.exe` `.msi` |
| macOS | `.dmg` `.app` |
| Linux | `.deb` `.AppImage` |

## 主な機能

- グローバルショートカット対応
- システムトレイ常駐
- クリップボード履歴の暗号化保存

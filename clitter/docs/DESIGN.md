# Clitter 設計書

## 1. 概要

### 1.1 プロジェクト名
Clitter - クリップボード管理システム

### 1.2 目的
クロスプラットフォームで動作するクリップボード履歴管理アプリケーションを構築する。
コピーした内容を自動分類し、ホワイトボード機能で整理・永続化できる。

### 1.3 技術スタック
| レイヤー | 技術 |
|---------|------|
| バックエンド | Rust (Tauri v2) |
| フロントエンド | Svelte 5 + TypeScript + Vite |
| 暗号化 | AES-256-GCM + Argon2id |
| データ保存 | SQLite (sqlx) + 暗号化対応 |
| スタイリング | Tailwind CSS |

---

## 2. システムアーキテクチャ

### 2.1 全体構成図

```
┌─────────────────────────────────────────────────────────────┐
│                      Clitter Application                    │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Frontend (Svelte + TypeScript)          │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌──────────────┐   │   │
│  │  │ ClipboardList│ │ Whiteboard  │ │ CategoryTabs │   │   │
│  │  └─────────────┘ └─────────────┘ └──────────────┘   │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌──────────────┐   │   │
│  │  │ StickyNote  │ │   Group     │ │PasswordModal│   │   │
│  │  └─────────────┘ └─────────────┘ └──────────────┘   │   │
│  │                                                      │   │
│  │  ┌─────────────────────────────────────────────┐    │   │
│  │  │              Stores (Svelte Store)           │    │   │
│  │  │  clipboard.ts  │  whiteboard.ts              │    │   │
│  │  └─────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────┘   │
│                            │                                │
│                    Tauri IPC Commands                       │
│                            │                                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │               Backend (Rust + Tauri)                 │   │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────────────┐  │   │
│  │  │ clipboard │ │  storage  │ │      crypto       │  │   │
│  │  │ ├─monitor │ │ ├─volatile│ │ └─secure_store    │  │   │
│  │  │ └─catego- │ │ └─persist-│ │                   │  │   │
│  │  │   rizer   │ │   ent     │ │                   │  │   │
│  │  └───────────┘ └───────────┘ └───────────────────┘  │   │
│  │  ┌───────────┐ ┌─────────────────────────────────┐  │   │
│  │  │  hotkey   │ │          commands.rs            │  │   │
│  │  └───────────┘ └─────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 ディレクトリ構成

```
clitter/
├── docs/
│   └── DESIGN.md              # 本設計書
├── src-tauri/
│   ├── Cargo.toml             # Rust依存関係
│   ├── tauri.conf.json        # Tauri設定
│   ├── capabilities/
│   │   └── default.json       # 権限設定
│   ├── migrations/            # SQLiteマイグレーション
│   │   └── 001_initial.sql
│   └── src/
│       ├── main.rs            # エントリーポイント
│       ├── lib.rs             # モジュール定義
│       ├── commands.rs        # Tauri IPCコマンド
│       ├── clipboard/
│       │   ├── mod.rs
│       │   ├── monitor.rs     # クリップボード監視
│       │   └── categorizer.rs # 自動カテゴリ分類
│       ├── storage/
│       │   ├── mod.rs
│       │   ├── volatile.rs    # 揮発性ストレージ
│       │   └── persistent.rs  # 永続化ストレージ
│       ├── crypto/
│       │   ├── mod.rs
│       │   └── secure_store.rs# 暗号化処理
│       └── hotkey/
│           └── mod.rs         # グローバルホットキー
├── src/
│   ├── App.svelte             # ルートコンポーネント
│   ├── main.ts                # エントリーポイント
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ClipboardList.svelte
│   │   │   ├── ClipboardItem.svelte
│   │   │   ├── Whiteboard.svelte
│   │   │   ├── StickyNote.svelte
│   │   │   ├── Group.svelte
│   │   │   ├── CategoryTabs.svelte
│   │   │   └── PasswordModal.svelte
│   │   ├── stores/
│   │   │   ├── clipboard.ts
│   │   │   └── whiteboard.ts
│   │   └── types/
│   │       └── index.ts
│   └── styles/
│       └── global.css
├── package.json
├── vite.config.ts
├── tailwind.config.js
└── tsconfig.json
```

---

## 3. データモデル

### 3.1 TypeScript 型定義

```typescript
// カテゴリ種別
type Category = "text" | "image" | "numeric" | "secure";

// クリップボードコンテンツ
interface ClipboardContent {
  id: string;           // UUID v4
  category: Category;
  data: TextData | ImageData;
  copiedAt: string;     // ISO 8601
  source?: string;      // コピー元アプリ名 (取得可能な場合)
}

interface TextData {
  type: "text";
  text: string;
  preview: string;      // 最初の100文字
}

interface ImageData {
  type: "image";
  base64: string;       // Base64エンコード画像
  width: number;
  height: number;
  format: string;       // png, jpg, etc.
}

// ホワイトボードアイテム (付箋)
interface WhiteboardItem {
  id: string;           // UUID v4
  content: ClipboardContent;
  position: { x: number; y: number };
  size: { width: number; height: number };
  parentGroup: string | null;  // グループID
  shortcut: string | null;     // カスタムショートカット (例: "a", "b", "ctrl+1")
  createdAt: string;
  updatedAt: string;
}

// グループ (アコーディオン)
interface Group {
  id: string;           // UUID v4
  name: string;
  position: { x: number; y: number };
  collapsed: boolean;
  parentGroup: string | null;  // 親グループID (階層対応)
  children: string[];   // 子アイテム/グループのID
  shortcut: string | null;     // カスタムショートカット
  createdAt: string;
  updatedAt: string;
}

// ホワイトボード全体
interface WhiteboardState {
  items: Record<string, WhiteboardItem>;
  groups: Record<string, Group>;
  rootItems: string[];  // ルートレベルのアイテムID
}
```

### 3.2 Rust 型定義

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Text,
    Image,
    Numeric,
    Secure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClipboardData {
    Text {
        text: String,
        preview: String,
    },
    Image {
        base64: String,
        width: u32,
        height: u32,
        format: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardContent {
    pub id: Uuid,
    pub category: Category,
    pub data: ClipboardData,
    pub copied_at: DateTime<Utc>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardItem {
    pub id: Uuid,
    pub content: ClipboardContent,
    pub position: Position,
    pub size: Size,
    pub parent_group: Option<Uuid>,
    pub shortcut: Option<String>,  // カスタムショートカット
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub position: Position,
    pub collapsed: bool,
    pub parent_group: Option<Uuid>,
    pub children: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## 4. 機能仕様

### 4.1 ホットキー

#### グローバルホットキー (システム全体)
| キー | 機能 | 説明 |
|------|------|------|
| `Alt + V` | ウィンドウ表示/非表示 | トグル動作、表示時にフォーカス |

#### ウィンドウ内ホットキー (アプリ表示中のみ)
| キー | 機能 | 説明 |
|------|------|------|
| `←` | 画像カテゴリ選択 | フィルタ切替 |
| `↓` | 文字カテゴリ選択 | フィルタ切替 |
| `→` | 数字系カテゴリ選択 | フィルタ切替 |
| `↑` | ビジュアル機能 | ホワイトボード表示/操作 |
| `1` ~ `5` | 履歴クイックアクセス | 直近1~5番目のアイテムをクリップボードにコピー |
| `Esc` | ウィンドウ非表示 | 閉じる |

#### ホワイトボード内カスタムショートカット (インクリメンタル入力)
ホワイトボードの付箋・グループに任意のショートカット文字列を割り当て可能。
ホワイトボード表示中にキー入力するとインクリメンタル検索でマッチング。

**操作フロー例1**: 単一アイテム
```
Alt+V → ↑ → "da" → Enter
                    └── "da"がマッチしたアイテムをコピー&終了
```

**操作フロー例2**: グループ + 子アイテム
```
Alt+V → ↑ → "da" → Space → "dev" → Enter
            │        │       │        └── "dev"アイテムをコピー
            │        │       └── 子アイテム"dev"を入力
            │        └── グループ展開
            └── グループ"da"にマッチ
```

| キー | 機能 |
|------|------|
| 英数字入力 | ショートカットのインクリメンタル検索 |
| `Enter` | マッチしたアイテムをコピー&ウィンドウ非表示 |
| `Space` | マッチしたグループを展開/折りたたみ |
| `Backspace` | 入力を1文字削除 |
| `Esc` | 入力クリア / ホワイトボード閉じる |

**割り当て方法**: 付箋/グループを右クリック → 「ショートカット設定」→ 文字列入力

**設定例**:
| 対象 | ショートカット | 備考 |
|------|---------------|------|
| グループ「データベース接続」 | `da` | |
| └ 付箋「開発環境」 | `dev` | グループ内のみ有効 |
| └ 付箋「ステージング」 | `stg` | |
| └ 付箋「本番環境」 | `prd` | |
| 付箋「APIキー」 | `api` | ルートレベル |

**実装方式**:
- グローバル: `tauri-plugin-global-shortcut`
- ウィンドウ内: Svelte の `keydown` イベント

```rust
// hotkey/mod.rs
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

pub fn register_global_shortcuts(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Alt+V: ウィンドウ表示/非表示
    let toggle_shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyV);

    app.global_shortcut().on_shortcut(toggle_shortcut, |app, _| {
        if let Some(window) = app.get_webview_window("main") {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    })?;

    Ok(())
}
```

```typescript
// App.svelte - ウィンドウ内キーボード操作
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

function handleKeydown(event: KeyboardEvent) {
  switch (event.key) {
    case 'ArrowLeft':
      selectedCategory.set('image');
      break;
    case 'ArrowDown':
      selectedCategory.set('text');
      break;
    case 'ArrowRight':
      selectedCategory.set('numeric');
      break;
    case 'ArrowUp':
      // ビジュアル機能 (ホワイトボード) 表示
      currentView.set('whiteboard');
      break;
    case '1': case '2': case '3': case '4': case '5':
      // 番号キーで直近履歴をクリップボードにコピー
      const index = parseInt(event.key) - 1;
      copyHistoryItem(index);
      break;
    case 'Escape':
      getCurrentWindow().hide();
      break;
  }
}

async function copyHistoryItem(index: number) {
  const items = get(filteredHistory);
  if (index < items.length) {
    await invoke('copy_to_clipboard', { content: items[index] });
    // コピー後にウィンドウを非表示
    getCurrentWindow().hide();
  }
}
```

### 4.2 クリップボード監視

**監視方式**: `arboard` クレートを使用した1秒間隔ポーリング

```rust
// clipboard/monitor.rs
use arboard::Clipboard;
use std::time::Duration;
use tokio::time::interval;

pub struct ClipboardMonitor {
    clipboard: Clipboard,
    last_content_hash: Option<u64>,
}

impl ClipboardMonitor {
    pub async fn start_monitoring<F>(&mut self, on_change: F)
    where
        F: Fn(ClipboardContent) + Send + 'static,
    {
        let mut interval = interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            if let Some(content) = self.check_clipboard_change() {
                on_change(content);
            }
        }
    }
}
```

### 4.3 自動カテゴリ分類

**分類ロジック**:

```rust
// clipboard/categorizer.rs
use regex::Regex;

pub struct Categorizer {
    numeric_pattern: Regex,
    secure_patterns: Vec<Regex>,
}

impl Categorizer {
    pub fn new() -> Self {
        Self {
            // 数字系: 数字、カンマ、ピリオド、ハイフン、スラッシュで構成
            numeric_pattern: Regex::new(r"^[\d,.\-/\s]+$").unwrap(),

            // セキュア情報パターン
            secure_patterns: vec![
                Regex::new(r"(?i)(api[_-]?key|apikey)").unwrap(),
                Regex::new(r"(?i)(password|passwd|pwd)").unwrap(),
                Regex::new(r"(?i)(secret|token|bearer)").unwrap(),
                Regex::new(r"(?i)(private[_-]?key)").unwrap(),
                Regex::new(r"^[A-Za-z0-9+/]{32,}={0,2}$").unwrap(), // Base64-like
                Regex::new(r"^[a-f0-9]{32,}$").unwrap(), // Hex string (API keys)
            ],
        }
    }

    pub fn categorize(&self, content: &ClipboardData) -> Category {
        match content {
            ClipboardData::Image { .. } => Category::Image,
            ClipboardData::Text { text, .. } => {
                // セキュア情報チェック
                for pattern in &self.secure_patterns {
                    if pattern.is_match(text) {
                        return Category::Secure;
                    }
                }

                // 数字系チェック
                if self.numeric_pattern.is_match(text.trim()) {
                    return Category::Numeric;
                }

                Category::Text
            }
        }
    }
}
```

**分類フローチャート**:

```
クリップボードデータ
        │
        ▼
   ┌─────────┐
   │ 画像？  │──Yes──▶ Category::Image (←キー)
   └────┬────┘
        │No
        ▼
   ┌─────────────┐
   │セキュアパタ │──Yes──▶ Category::Secure (自動検出、🔒アイコン表示)
   │ーンマッチ？ │
   └────┬────────┘
        │No
        ▼
   ┌─────────┐
   │ 数字系？ │──Yes──▶ Category::Numeric (→キー)
   └────┬────┘
        │No
        ▼
   Category::Text (↓キー)
```

**注意**: セキュア情報カテゴリはホットキーでは選択不可。自動検出のみで、
履歴リスト内では🔒アイコンで識別。タブクリックで表示可能。

### 4.4 揮発性ストレージ

**特徴**:
- メモリ上に最大100件保持
- VecDeque でFIFO管理
- アプリ終了時に破棄

```rust
// storage/volatile.rs
use std::collections::VecDeque;
use tokio::sync::RwLock;

const MAX_HISTORY_SIZE: usize = 100;

pub struct VolatileStorage {
    history: RwLock<VecDeque<ClipboardContent>>,
}

impl VolatileStorage {
    pub fn new() -> Self {
        Self {
            history: RwLock::new(VecDeque::with_capacity(MAX_HISTORY_SIZE)),
        }
    }

    pub async fn add(&self, content: ClipboardContent) {
        let mut history = self.history.write().await;

        // 重複チェック (直前と同じ内容はスキップ)
        if let Some(last) = history.front() {
            if last.id == content.id {
                return;
            }
        }

        // 容量超過時は古いものを削除
        if history.len() >= MAX_HISTORY_SIZE {
            history.pop_back();
        }

        history.push_front(content);
    }

    pub async fn get_by_category(&self, category: Option<Category>) -> Vec<ClipboardContent> {
        let history = self.history.read().await;

        match category {
            Some(cat) => history.iter()
                .filter(|c| c.category == cat)
                .cloned()
                .collect(),
            None => history.iter().cloned().collect(),
        }
    }

    pub async fn get_recent(&self, count: usize) -> Vec<ClipboardContent> {
        let history = self.history.read().await;
        history.iter().take(count).cloned().collect()
    }
}
```

### 4.5 ホワイトボード (永続化 - SQLite)

**保存場所**: `~/.clitter/clitter.db`

**データベーススキーマ**:

```sql
-- クリップボードコンテンツ (ホワイトボード用)
CREATE TABLE clipboard_contents (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL CHECK (category IN ('text', 'image', 'numeric', 'secure')),
    data_type TEXT NOT NULL CHECK (data_type IN ('text', 'image')),
    text_content TEXT,
    text_preview TEXT,
    image_base64 TEXT,
    image_width INTEGER,
    image_height INTEGER,
    image_format TEXT,
    source TEXT,
    copied_at TEXT NOT NULL,
    is_encrypted INTEGER DEFAULT 0
);

-- ホワイトボードアイテム (付箋)
CREATE TABLE whiteboard_items (
    id TEXT PRIMARY KEY,
    content_id TEXT NOT NULL REFERENCES clipboard_contents(id) ON DELETE CASCADE,
    position_x REAL NOT NULL,
    position_y REAL NOT NULL,
    width REAL NOT NULL DEFAULT 200,
    shortcut TEXT,  -- カスタムショートカット (例: "a", "ctrl+1")
    height REAL NOT NULL DEFAULT 150,
    parent_group_id TEXT REFERENCES groups(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- グループ (アコーディオン)
CREATE TABLE groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    position_x REAL NOT NULL,
    position_y REAL NOT NULL,
    collapsed INTEGER NOT NULL DEFAULT 0,
    parent_group_id TEXT REFERENCES groups(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 暗号化設定
CREATE TABLE encryption_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    salt BLOB NOT NULL,
    verification_hash TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- インデックス
CREATE INDEX idx_whiteboard_items_parent ON whiteboard_items(parent_group_id);
CREATE INDEX idx_groups_parent ON groups(parent_group_id);
CREATE INDEX idx_clipboard_contents_category ON clipboard_contents(category);
```

```rust
// storage/persistent.rs
use directories::ProjectDirs;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::PathBuf;

pub struct PersistentStorage {
    pool: Pool<Sqlite>,
    data_dir: PathBuf,
}

impl PersistentStorage {
    pub async fn new() -> Result<Self, StorageError> {
        let dirs = ProjectDirs::from("com", "clitter", "Clitter")
            .ok_or(StorageError::NoHomeDir)?;

        let data_dir = dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("clitter.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool, data_dir })
    }

    pub async fn save_whiteboard_item(&self, item: &WhiteboardItem) -> Result<(), StorageError> {
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO whiteboard_items
            (id, content_id, position_x, position_y, width, height, parent_group_id, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            item.id,
            item.content.id,
            item.position.x,
            item.position.y,
            item.size.width,
            item.size.height,
            item.parent_group,
            item.created_at,
            item.updated_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn load_whiteboard(&self) -> Result<WhiteboardState, StorageError> {
        let items = sqlx::query_as!(
            WhiteboardItemRow,
            "SELECT * FROM whiteboard_items"
        )
        .fetch_all(&self.pool)
        .await?;

        let groups = sqlx::query_as!(
            GroupRow,
            "SELECT * FROM groups"
        )
        .fetch_all(&self.pool)
        .await?;

        // Convert to WhiteboardState
        Ok(WhiteboardState::from_rows(items, groups))
    }

    pub async fn delete_whiteboard_item(&self, id: &Uuid) -> Result<(), StorageError> {
        sqlx::query!("DELETE FROM whiteboard_items WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
```

### 4.6 暗号化モジュール

**アルゴリズム**:
- 鍵導出: Argon2id (メモリ64MB, 反復3回, 並列4)
- 暗号化: AES-256-GCM

**保存形式**:
```
[salt: 32bytes][nonce: 12bytes][ciphertext: variable][tag: 16bytes]
```

```rust
// crypto/secure_store.rs
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, Algorithm, Params, Version};
use rand::RngCore;

const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;

pub struct SecureStore {
    key: [u8; 32],
}

impl SecureStore {
    pub fn from_password(password: &str, salt: &[u8; SALT_LEN]) -> Result<Self, CryptoError> {
        let params = Params::new(65536, 3, 4, Some(32))?; // 64MB, 3 iterations, 4 parallelism
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut key = [0u8; 32];
        argon2.hash_password_into(password.as_bytes(), salt, &mut key)?;

        Ok(Self { key })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)?;

        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext)?;

        // Format: nonce || ciphertext
        let mut result = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if data.len() < NONCE_LEN {
            return Err(CryptoError::InvalidData);
        }

        let cipher = Aes256Gcm::new_from_slice(&self.key)?;
        let nonce = Nonce::from_slice(&data[..NONCE_LEN]);
        let ciphertext = &data[NONCE_LEN..];

        let plaintext = cipher.decrypt(nonce, ciphertext)?;
        Ok(plaintext)
    }
}
```

---

## 5. Tauri IPC コマンド

### 5.1 コマンド一覧

```rust
// commands.rs

/// クリップボード履歴を取得
#[tauri::command]
async fn get_clipboard_history(
    category: Option<Category>,
    state: State<'_, AppState>,
) -> Result<Vec<ClipboardContent>, String>;

/// 最近のクリップボードアイテムを取得
#[tauri::command]
async fn get_recent_items(
    count: usize,
    state: State<'_, AppState>,
) -> Result<Vec<ClipboardContent>, String>;

/// アイテムをクリップボードにコピー
#[tauri::command]
async fn copy_to_clipboard(
    content: ClipboardContent,
    state: State<'_, AppState>,
) -> Result<(), String>;

/// ホワイトボード状態を取得
#[tauri::command]
async fn get_whiteboard(
    state: State<'_, AppState>,
) -> Result<WhiteboardState, String>;

/// ホワイトボードにアイテム追加
#[tauri::command]
async fn add_to_whiteboard(
    content: ClipboardContent,
    position: Position,
    state: State<'_, AppState>,
) -> Result<WhiteboardItem, String>;

/// ホワイトボードアイテムを更新
#[tauri::command]
async fn update_whiteboard_item(
    id: Uuid,
    position: Option<Position>,
    size: Option<Size>,
    parent_group: Option<Uuid>,
    state: State<'_, AppState>,
) -> Result<(), String>;

/// ホワイトボードからアイテム削除
#[tauri::command]
async fn remove_from_whiteboard(
    id: Uuid,
    state: State<'_, AppState>,
) -> Result<(), String>;

/// グループ作成
#[tauri::command]
async fn create_group(
    name: String,
    position: Position,
    state: State<'_, AppState>,
) -> Result<Group, String>;

/// グループ更新
#[tauri::command]
async fn update_group(
    id: Uuid,
    name: Option<String>,
    collapsed: Option<bool>,
    position: Option<Position>,
    state: State<'_, AppState>,
) -> Result<(), String>;

/// グループ削除
#[tauri::command]
async fn delete_group(
    id: Uuid,
    state: State<'_, AppState>,
) -> Result<(), String>;

/// マスターパスワード設定
#[tauri::command]
async fn set_master_password(
    password: String,
    state: State<'_, AppState>,
) -> Result<(), String>;

/// マスターパスワード検証
#[tauri::command]
async fn verify_master_password(
    password: String,
    state: State<'_, AppState>,
) -> Result<bool, String>;

/// カテゴリ変更イベント発行
#[tauri::command]
async fn change_category(
    category: Category,
    app: AppHandle,
) -> Result<(), String>;
```

### 5.2 イベント一覧

| イベント名 | 方向 | ペイロード | 説明 |
|-----------|------|-----------|------|
| `clipboard-changed` | Backend → Frontend | `ClipboardContent` | 新しいクリップボードアイテム検出 |
| `category-changed` | Backend → Frontend | `Category` | ホットキーによるカテゴリ変更 |
| `whiteboard-updated` | Backend → Frontend | `WhiteboardState` | ホワイトボード状態変更 |

---

## 6. フロントエンドコンポーネント

### 6.1 コンポーネント階層

```
App.svelte
├── CategoryTabs.svelte          # カテゴリ切替タブ
├── ClipboardList.svelte         # 履歴リスト
│   └── ClipboardItem.svelte     # 個別アイテム (番号バッジ付き)
├── Whiteboard.svelte            # ホワイトボード (ビジュアル機能)
│   ├── StickyNote.svelte        # 付箋
│   └── Group.svelte             # グループ (アコーディオン)
│       ├── StickyNote.svelte
│       └── Group.svelte         # ネスト可能
└── PasswordModal.svelte         # パスワード入力モーダル
```

**ClipboardItem.svelte - 番号バッジ表示**
```svelte
<script lang="ts">
  export let item: ClipboardContent;
  export let index: number;  // 0-4

  const numberBadges = ['①', '②', '③', '④', '⑤'];
  const categoryIcons = {
    text: '📋',
    image: '🖼️',
    numeric: '🔢',
    secure: '🔒'
  };
</script>

<div class="clipboard-item" role="button" tabindex="0">
  <span class="number-badge">{numberBadges[index]}</span>
  <span class="category-icon">{categoryIcons[item.category]}</span>
  <span class="preview">
    {#if item.category === 'secure'}
      ••••••••••••
    {:else if item.data.type === 'image'}
      <img src={`data:image/${item.data.format};base64,${item.data.base64}`} alt="thumbnail" />
    {:else}
      {item.data.preview}
    {/if}
  </span>
</div>

<style>
  .number-badge {
    @apply text-primary-600 font-bold mr-2;
  }
</style>
```

### 6.2 Svelte Store

```typescript
// stores/clipboard.ts
import { writable, derived } from 'svelte/store';
import type { ClipboardContent, Category } from '$lib/types';

export const clipboardHistory = writable<ClipboardContent[]>([]);
export const selectedCategory = writable<Category | null>(null);

export const filteredHistory = derived(
  [clipboardHistory, selectedCategory],
  ([$history, $category]) => {
    if ($category === null) {
      return $history.slice(0, 5); // 直近5件
    }
    return $history.filter(item => item.category === $category);
  }
);

// stores/ui.ts
import { writable } from 'svelte/store';

// 画面表示状態
export type ViewMode = 'list' | 'whiteboard';
export const currentView = writable<ViewMode>('list');

// stores/whiteboard.ts
import { writable } from 'svelte/store';
import type { WhiteboardState } from '$lib/types';

export const whiteboardState = writable<WhiteboardState>({
  items: {},
  groups: {},
  rootItems: [],
});

export const draggedItem = writable<string | null>(null);
```

### 6.3 UI レイアウト

**メイン画面 (履歴リスト表示)**
```
┌────────────────────────────────────────┐
│  Clitter                          [×]  │
├────────────────────────────────────────┤
│  [← 画像] [↓ 文字] [→ 数字] [🔒]      │
│  ─────────────────────────────────────  │
│  ┌──────────────────────────────────┐  │
│  │ ① 📋 コピーしたテキストのプレ... │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │ ② 🖼️ [画像サムネイル]           │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │ ③ 🔢 1,234,567                   │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │ ④ 🔒 ••••••••••••                │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │ ⑤ 📋 別のテキスト...             │  │
│  └──────────────────────────────────┘  │
│                                        │
│  ─────────────────────────────────────  │
│  ↑ ホワイトボードを開く               │
└────────────────────────────────────────┘

※ 番号キー(1~5)で即座にコピー&ウィンドウ非表示
```

**ホワイトボード画面 (↑キーで表示)**
```
┌────────────────────────────────────────────────────────────────┐
│  🎯 ホワイトボード                              [← 戻る]      │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│    ┌─────────┐                                                │
│    │ 付箋 1  │      ┌─────────┐                              │
│    │         │      │ 付箋 2  │                              │
│    └─────────┘      └─────────┘                              │
│                                                                │
│    ┌──────────────────────┐                                   │
│    │ 📁 グループA (▼展開) │                                   │
│    │  ┌─────────┐         │                                   │
│    │  │ 付箋 3  │         │                                   │
│    │  └─────────┘         │                                   │
│    │  ┌─────────┐         │                                   │
│    │  │ 付箋 4  │         │                                   │
│    │  └─────────┘         │                                   │
│    └──────────────────────┘                                   │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

**操作フロー**:
1. `Alt+V` → メイン画面表示
2. `1`~`5` → 直近履歴を即コピー&非表示 (最速操作)
3. `←↓→` → カテゴリ切替 (画像/文字/数字)
4. `↑` → ホワイトボード画面へ
5. `Esc` or `←戻る` → 前の画面/非表示

**典型的な使用パターン**:
- `Alt+V` → `1` : 直前にコピーしたものを再度コピー
- `Alt+V` → `3` : 3番目の履歴をコピー
- `Alt+V` → `←` → `1` : 直近の画像をコピー

---

## 7. セキュリティ考慮事項

### 7.1 セキュア情報の保護

1. **メモリ上の保護**
   - セキュア情報は暗号化状態で保持
   - 表示時のみ復号
   - マスク表示オプション

2. **永続化時の保護**
   - ホワイトボード内のセキュア情報は暗号化して保存
   - マスターパスワード未設定時はセキュア情報の永続化を警告

3. **クリップボード経由の漏洩防止**
   - セキュア情報コピー時に確認ダイアログ
   - 一定時間後にクリップボードをクリアするオプション

### 7.2 パスワード管理

- マスターパスワードはメモリ上に保持しない
- セッション中は導出済みの鍵のみ保持
- アプリ終了時に鍵を破棄

---

## 8. 実装フェーズ

### Phase 1: プロジェクト基盤 (優先度: 高)
- [x] プロジェクト構成設計
- [ ] Tauri + Svelte プロジェクト初期化
- [ ] 基本ウィンドウ設定
- [ ] グローバルホットキー (Alt+V)

### Phase 2: クリップボード機能 (優先度: 高)
- [ ] クリップボード監視実装
- [ ] カテゴリ自動分類
- [ ] 揮発性ストレージ
- [ ] 履歴リストUI

### Phase 3: カテゴリ切替 (優先度: 中)
- [ ] Alt + 矢印キーでカテゴリ切替
- [ ] カテゴリ別表示UI
- [ ] カテゴリタブコンポーネント

### Phase 4: ホワイトボード (優先度: 中)
- [ ] ホワイトボードUI基盤
- [ ] 付箋ドラッグ&ドロップ
- [ ] グループ機能 (アコーディオン)
- [ ] 階層グループ対応
- [ ] SQLite永続化

### Phase 5: セキュリティ (優先度: 中)
- [ ] 暗号化モジュール実装
- [ ] マスターパスワード入力UI
- [ ] セキュア情報の暗号化保存

### Phase 6: 仕上げ (優先度: 低)
- [ ] クロスプラットフォームテスト
- [ ] パフォーマンス最適化
- [ ] アイコン・UI調整

---

## 9. 依存関係

### 9.1 Rust (Cargo.toml)

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-shell = "2"
arboard = "3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full", "sync"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
aes-gcm = "0.10"
argon2 = "0.5"
rand = "0.8"
base64 = "0.22"
regex = "1"
thiserror = "1"
directories = "5"
once_cell = "1"
```

### 9.2 JavaScript (package.json)

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-global-shortcut": "^2.0.0"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^4.0.0",
    "@tauri-apps/cli": "^2.0.0",
    "svelte": "^5.0.0",
    "tailwindcss": "^3.4.0",
    "typescript": "^5.3.0",
    "vite": "^6.0.0"
  }
}
```

---

## 10. テスト計画

### 10.1 ユニットテスト

| モジュール | テスト内容 |
|-----------|-----------|
| categorizer | 各カテゴリの分類精度 |
| volatile_storage | 追加・削除・容量制限 |
| persistent_storage | SQLite CRUD操作、マイグレーション |
| crypto | 暗号化・復号の正確性 |

### 10.2 統合テスト

1. クリップボードコピー → 履歴追加 → カテゴリ分類
2. 履歴からホワイトボードへドラッグ&ドロップ
3. アプリ再起動後のホワイトボード復元
4. 暗号化保存 → 再起動 → パスワード入力 → 復号

### 10.3 E2Eテスト

```bash
# 開発サーバー起動
cargo tauri dev

# テスト手順
1. Alt+V でウィンドウ表示確認
2. テキストコピー → 履歴に追加されることを確認
3. 各カテゴリ切替が機能することを確認
4. ホワイトボードへのドラッグ&ドロップ確認
5. アプリ再起動後もホワイトボードデータが復元されることを確認
```

---

## 改訂履歴

| 版 | 日付 | 内容 |
|----|------|------|
| 1.0 | 2026-01-27 | 初版作成 |

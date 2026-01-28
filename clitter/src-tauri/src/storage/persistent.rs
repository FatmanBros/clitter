use directories::ProjectDirs;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

use crate::types::{
    Category, ClipboardContent, ClipboardData, Group, Position, Size, WhiteboardItem,
    WhiteboardState,
};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("No home directory found")]
    NoHomeDir,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct PersistentStorage {
    pool: Pool<Sqlite>,
    #[allow(dead_code)]
    data_dir: PathBuf,
}

impl PersistentStorage {
    pub async fn new() -> Result<Self, StorageError> {
        let dirs =
            ProjectDirs::from("com", "clitter", "Clitter").ok_or(StorageError::NoHomeDir)?;

        let data_dir = dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("clitter.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // Run migrations
        Self::run_migrations(&pool).await?;

        Ok(Self { pool, data_dir })
    }

    async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS clipboard_contents (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                data_type TEXT NOT NULL,
                text_content TEXT,
                text_preview TEXT,
                image_base64 TEXT,
                image_width INTEGER,
                image_height INTEGER,
                image_format TEXT,
                source TEXT,
                copied_at TEXT NOT NULL,
                is_encrypted INTEGER DEFAULT 0
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS whiteboard_items (
                id TEXT PRIMARY KEY,
                content_id TEXT NOT NULL,
                position_x REAL NOT NULL,
                position_y REAL NOT NULL,
                width REAL NOT NULL DEFAULT 200,
                height REAL NOT NULL DEFAULT 150,
                parent_group_id TEXT,
                shortcut TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (content_id) REFERENCES clipboard_contents(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                position_x REAL NOT NULL,
                position_y REAL NOT NULL,
                collapsed INTEGER NOT NULL DEFAULT 0,
                parent_group_id TEXT,
                shortcut TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (parent_group_id) REFERENCES groups(id) ON DELETE SET NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS encryption_config (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                salt BLOB NOT NULL,
                verification_hash TEXT NOT NULL,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Create indexes
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_whiteboard_items_parent ON whiteboard_items(parent_group_id)",
        )
        .execute(pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_groups_parent ON groups(parent_group_id)")
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn save_clipboard_content(
        &self,
        content: &ClipboardContent,
    ) -> Result<(), StorageError> {
        let (data_type, text_content, text_preview, image_base64, image_width, image_height, image_format) =
            match &content.data {
                ClipboardData::Text { text, preview } => {
                    ("text", Some(text.clone()), Some(preview.clone()), None, None, None, None)
                }
                ClipboardData::Image {
                    base64,
                    width,
                    height,
                    format,
                } => (
                    "image",
                    None,
                    None,
                    Some(base64.clone()),
                    Some(*width as i64),
                    Some(*height as i64),
                    Some(format.clone()),
                ),
            };

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO clipboard_contents
            (id, category, data_type, text_content, text_preview, image_base64, image_width, image_height, image_format, source, copied_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(content.id.to_string())
        .bind(content.category.to_string())
        .bind(data_type)
        .bind(text_content)
        .bind(text_preview)
        .bind(image_base64)
        .bind(image_width)
        .bind(image_height)
        .bind(image_format)
        .bind(&content.source)
        .bind(content.copied_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_whiteboard_item(&self, item: &WhiteboardItem) -> Result<(), StorageError> {
        // First save the content
        self.save_clipboard_content(&item.content).await?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO whiteboard_items
            (id, content_id, position_x, position_y, width, height, parent_group_id, shortcut, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(item.id.to_string())
        .bind(item.content.id.to_string())
        .bind(item.position.x)
        .bind(item.position.y)
        .bind(item.size.width)
        .bind(item.size.height)
        .bind(item.parent_group.map(|id| id.to_string()))
        .bind(&item.shortcut)
        .bind(item.created_at.to_rfc3339())
        .bind(item.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_group(&self, group: &Group) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO groups
            (id, name, position_x, position_y, collapsed, parent_group_id, shortcut, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(group.id.to_string())
        .bind(&group.name)
        .bind(group.position.x)
        .bind(group.position.y)
        .bind(group.collapsed)
        .bind(group.parent_group.map(|id| id.to_string()))
        .bind(&group.shortcut)
        .bind(group.created_at.to_rfc3339())
        .bind(group.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_whiteboard(&self) -> Result<WhiteboardState, StorageError> {
        let mut state = WhiteboardState::default();

        // Load groups
        let group_rows = sqlx::query(
            "SELECT id, name, position_x, position_y, collapsed, parent_group_id, shortcut, created_at, updated_at FROM groups",
        )
        .fetch_all(&self.pool)
        .await?;

        for row in group_rows {
            let id: String = row.get("id");
            let id = Uuid::parse_str(&id).unwrap_or_default();

            let parent_group: Option<String> = row.get("parent_group_id");
            let parent_group = parent_group.and_then(|s| Uuid::parse_str(&s).ok());

            let group = Group {
                id,
                name: row.get("name"),
                position: Position {
                    x: row.get("position_x"),
                    y: row.get("position_y"),
                },
                collapsed: row.get::<i32, _>("collapsed") != 0,
                parent_group,
                children: Vec::new(),
                shortcut: row.get("shortcut"),
                created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            };

            state.groups.insert(id, group);
        }

        // Load whiteboard items with their content
        let item_rows = sqlx::query(
            r#"
            SELECT
                wi.id, wi.position_x, wi.position_y, wi.width, wi.height, wi.parent_group_id, wi.shortcut, wi.created_at, wi.updated_at,
                cc.id as content_id, cc.category, cc.data_type, cc.text_content, cc.text_preview,
                cc.image_base64, cc.image_width, cc.image_height, cc.image_format, cc.source, cc.copied_at
            FROM whiteboard_items wi
            JOIN clipboard_contents cc ON wi.content_id = cc.id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for row in item_rows {
            let id: String = row.get("id");
            let id = Uuid::parse_str(&id).unwrap_or_default();

            let content_id: String = row.get("content_id");
            let content_id = Uuid::parse_str(&content_id).unwrap_or_default();

            let parent_group: Option<String> = row.get("parent_group_id");
            let parent_group = parent_group.and_then(|s| Uuid::parse_str(&s).ok());

            let category: String = row.get("category");
            let category: Category = category.parse().unwrap_or(Category::Text);

            let data_type: String = row.get("data_type");
            let data = if data_type == "image" {
                ClipboardData::Image {
                    base64: row.get::<Option<String>, _>("image_base64").unwrap_or_default(),
                    width: row.get::<Option<i64>, _>("image_width").unwrap_or(0) as u32,
                    height: row.get::<Option<i64>, _>("image_height").unwrap_or(0) as u32,
                    format: row.get::<Option<String>, _>("image_format").unwrap_or_default(),
                }
            } else {
                ClipboardData::Text {
                    text: row.get::<Option<String>, _>("text_content").unwrap_or_default(),
                    preview: row.get::<Option<String>, _>("text_preview").unwrap_or_default(),
                }
            };

            let content = ClipboardContent {
                id: content_id,
                category,
                data,
                copied_at: chrono::DateTime::parse_from_rfc3339(row.get("copied_at"))
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                source: row.get("source"),
            };

            let item = WhiteboardItem {
                id,
                content,
                position: Position {
                    x: row.get("position_x"),
                    y: row.get("position_y"),
                },
                size: Size {
                    width: row.get("width"),
                    height: row.get("height"),
                },
                parent_group,
                shortcut: row.get("shortcut"),
                created_at: chrono::DateTime::parse_from_rfc3339(row.get("created_at"))
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                updated_at: chrono::DateTime::parse_from_rfc3339(row.get("updated_at"))
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
            };

            if parent_group.is_none() {
                state.root_items.push(id);
            }

            state.items.insert(id, item);
        }

        Ok(state)
    }

    pub async fn delete_whiteboard_item(&self, id: Uuid) -> Result<(), StorageError> {
        // Get content_id first
        let row = sqlx::query("SELECT content_id FROM whiteboard_items WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let content_id: String = row.get("content_id");

            // Delete whiteboard item
            sqlx::query("DELETE FROM whiteboard_items WHERE id = ?")
                .bind(id.to_string())
                .execute(&self.pool)
                .await?;

            // Delete content
            sqlx::query("DELETE FROM clipboard_contents WHERE id = ?")
                .bind(content_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    pub async fn delete_group(&self, id: Uuid) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM groups WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_item_shortcut(
        &self,
        id: Uuid,
        shortcut: Option<String>,
    ) -> Result<(), StorageError> {
        sqlx::query("UPDATE whiteboard_items SET shortcut = ?, updated_at = ? WHERE id = ?")
            .bind(shortcut)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_items_with_shortcuts(&self) -> Result<Vec<(String, Uuid)>, StorageError> {
        let rows = sqlx::query("SELECT id, shortcut FROM whiteboard_items WHERE shortcut IS NOT NULL")
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let shortcut: String = row.get("shortcut");
            if let Ok(uuid) = Uuid::parse_str(&id) {
                result.push((shortcut, uuid));
            }
        }

        Ok(result)
    }
}

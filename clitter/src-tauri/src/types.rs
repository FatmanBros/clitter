use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Text,
    Image,
    Numeric,
    Secure,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Text => write!(f, "text"),
            Category::Image => write!(f, "image"),
            Category::Numeric => write!(f, "numeric"),
            Category::Secure => write!(f, "secure"),
        }
    }
}

impl std::str::FromStr for Category {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(Category::Text),
            "image" => Ok(Category::Image),
            "numeric" => Ok(Category::Numeric),
            "secure" => Ok(Category::Secure),
            _ => Err(format!("Unknown category: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClipboardData {
    Text { text: String, preview: String },
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

impl ClipboardContent {
    pub fn new_text(text: String, category: Category) -> Self {
        let preview = text.chars().take(100).collect();
        Self {
            id: Uuid::new_v4(),
            category,
            data: ClipboardData::Text { text, preview },
            copied_at: Utc::now(),
            source: None,
        }
    }

    pub fn new_image(base64: String, width: u32, height: u32, format: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            category: Category::Image,
            data: ClipboardData::Image {
                base64,
                width,
                height,
                format,
            },
            copied_at: Utc::now(),
            source: None,
        }
    }

    pub fn content_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        match &self.data {
            ClipboardData::Text { text, .. } => text.hash(&mut hasher),
            ClipboardData::Image { base64, .. } => base64.hash(&mut hasher),
        }
        hasher.finish()
    }
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
pub struct WhiteboardItem {
    pub id: Uuid,
    pub content: ClipboardContent,
    pub position: Position,
    pub size: Size,
    pub parent_group: Option<Uuid>,
    pub shortcut: Option<String>,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WhiteboardItem {
    pub fn new(content: ClipboardContent, position: Position) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            content,
            position,
            size: Size {
                width: 120.0,
                height: 60.0,
            },
            parent_group: None,
            shortcut: None,
            label: None,
            created_at: now,
            updated_at: now,
        }
    }
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
    pub shortcut: Option<String>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Group {
    pub fn new(name: String, position: Position) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            position,
            collapsed: false,
            parent_group: None,
            children: Vec::new(),
            shortcut: None,
            color: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WhiteboardState {
    pub items: std::collections::HashMap<Uuid, WhiteboardItem>,
    pub groups: std::collections::HashMap<Uuid, Group>,
    pub root_items: Vec<Uuid>,
}

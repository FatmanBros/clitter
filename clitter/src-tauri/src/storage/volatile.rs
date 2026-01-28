use std::collections::VecDeque;
use tokio::sync::RwLock;

use crate::types::{Category, ClipboardContent};

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

        // Check for duplicate (same content hash as the most recent item)
        if let Some(last) = history.front() {
            if last.content_hash() == content.content_hash() {
                return;
            }
        }

        // Remove oldest if at capacity
        if history.len() >= MAX_HISTORY_SIZE {
            history.pop_back();
        }

        history.push_front(content);
    }

    pub async fn get_all(&self) -> Vec<ClipboardContent> {
        let history = self.history.read().await;
        history.iter().cloned().collect()
    }

    pub async fn get_by_category(&self, category: Category) -> Vec<ClipboardContent> {
        let history = self.history.read().await;
        history
            .iter()
            .filter(|c| c.category == category)
            .cloned()
            .collect()
    }

    pub async fn get_recent(&self, count: usize) -> Vec<ClipboardContent> {
        let history = self.history.read().await;
        history.iter().take(count).cloned().collect()
    }

    pub async fn get_by_index(&self, index: usize) -> Option<ClipboardContent> {
        let history = self.history.read().await;
        history.get(index).cloned()
    }

    pub async fn clear(&self) {
        let mut history = self.history.write().await;
        history.clear();
    }
}

impl Default for VolatileStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ClipboardData;

    fn create_test_content(text: &str) -> ClipboardContent {
        ClipboardContent {
            id: uuid::Uuid::new_v4(),
            category: Category::Text,
            data: ClipboardData::Text {
                text: text.to_string(),
                preview: text.to_string(),
            },
            copied_at: chrono::Utc::now(),
            source: None,
        }
    }

    #[tokio::test]
    async fn test_add_and_get() {
        let storage = VolatileStorage::new();

        let content = create_test_content("test");
        storage.add(content).await;

        let all = storage.get_all().await;
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_duplicate_prevention() {
        let storage = VolatileStorage::new();

        let content1 = create_test_content("same text");
        let content2 = create_test_content("same text");

        storage.add(content1).await;
        storage.add(content2).await;

        let all = storage.get_all().await;
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_max_capacity() {
        let storage = VolatileStorage::new();

        for i in 0..150 {
            let content = create_test_content(&format!("text {}", i));
            storage.add(content).await;
        }

        let all = storage.get_all().await;
        assert_eq!(all.len(), MAX_HISTORY_SIZE);
    }
}

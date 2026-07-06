use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub category: MemoryCategory,
    pub importance: f32,
    pub access_count: u32,
    pub last_accessed: u64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryCategory {
    Decision,
    Pattern,
    Error,
    Learning,
    Preference,
    Context,
}

pub struct LongTermMemory {
    entries: HashMap<String, MemoryEntry>,
    index: HashMap<MemoryCategory, Vec<String>>,
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            index: HashMap::new(),
        }
    }

    pub fn store(&mut self, key: &str, value: &str, category: MemoryCategory, importance: f32) {
        let entry = MemoryEntry {
            key: key.to_string(),
            value: value.to_string(),
            category: category.clone(),
            importance,
            access_count: 0,
            last_accessed: 0,
            tags: Vec::new(),
        };

        self.index.entry(category).or_default().push(key.to_string());
        self.entries.insert(key.to_string(), entry);
    }

    pub fn recall(&mut self, key: &str) -> Option<&MemoryEntry> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.access_count += 1;
            entry.last_accessed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Some(entry)
        } else {
            None
        }
    }

    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        let query_lower = query.to_lowercase();

        self.entries.values()
            .filter(|e| {
                e.key.to_lowercase().contains(&query_lower)
                    || e.value.to_lowercase().contains(&query_lower)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn by_category(&self, category: &MemoryCategory) -> Vec<&MemoryEntry> {
        self.index.get(category)
            .map(|keys| {
                keys.iter()
                    .filter_map(|k| self.entries.get(k))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn important(&self, min_importance: f32) -> Vec<&MemoryEntry> {
        self.entries.values()
            .filter(|e| e.importance >= min_importance)
            .collect()
    }

    pub fn recent(&self, count: usize) -> Vec<&MemoryEntry> {
        let mut entries: Vec<&MemoryEntry> = self.entries.values().collect();
        entries.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        entries.into_iter().take(count).collect()
    }

    pub fn frequent(&self, min_accesses: u32) -> Vec<&MemoryEntry> {
        self.entries.values()
            .filter(|e| e.access_count >= min_accesses)
            .collect()
    }

    pub fn prune(&mut self, max_entries: usize) {
        if self.entries.len() <= max_entries {
            return;
        }

        let mut entries: Vec<(String, f32)> = self.entries.iter()
            .map(|(k, e)| {
                let score = e.importance * 0.6 + (e.access_count as f32 * 0.1).min(0.4);
                (k.clone(), score)
            })
            .collect();

        entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let to_remove = entries.len() - max_entries;
        for (key, _) in entries.into_iter().take(to_remove) {
            self.entries.remove(&key);
            for keys in self.index.values_mut() {
                keys.retain(|k| k != &key);
            }
        }
    }

    pub fn stats(&self) -> MemoryStats {
        let total = self.entries.len();
        let total_accesses: u32 = self.entries.values().map(|e| e.access_count).sum();
        let avg_importance = if total > 0 {
            self.entries.values().map(|e| e.importance).sum::<f32>() / total as f32
        } else {
            0.0
        };

        MemoryStats {
            total_entries: total,
            total_accesses,
            avg_importance,
            categories: self.index.len(),
        }
    }

    pub fn export_json(&self) -> String {
        serde_json::to_string_pretty(&self.entries).unwrap_or_default()
    }

    pub fn import_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let entries: HashMap<String, MemoryEntry> = serde_json::from_str(json)?;
        for (key, entry) in entries {
            self.index.entry(entry.category.clone()).or_default().push(key.clone());
            self.entries.insert(key, entry);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub total_accesses: u32,
    pub avg_importance: f32,
    pub categories: usize,
}

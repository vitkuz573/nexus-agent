use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPalace {
    pub rooms: Vec<Room>,
    pub connections: Vec<Connection>,
    pub index: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub room_type: RoomType,
    pub items: Vec<MemoryItem>,
    pub importance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomType {
    Concept,
    Pattern,
    Decision,
    Bug,
    Learning,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    pub confidence: f32,
    pub access_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: String,
    pub to: String,
    pub strength: f32,
    pub relation: String,
}

impl MemoryPalace {
    pub fn new() -> Self {
        Self {
            rooms: Vec::new(),
            connections: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn add_room(&mut self, name: &str, room_type: RoomType) -> String {
        let id = format!("room-{}", self.rooms.len() + 1);
        let room = Room {
            id: id.clone(),
            name: name.to_string(),
            room_type,
            items: Vec::new(),
            importance: 0.5,
        };
        self.rooms.push(room);
        id
    }

    pub fn add_item(&mut self, room_id: &str, key: &str, value: &str, confidence: f32) {
        if let Some(room) = self.rooms.iter_mut().find(|r| r.id == room_id) {
            room.items.push(MemoryItem {
                key: key.to_string(),
                value: value.to_string(),
                confidence,
                access_count: 0,
            });

            self.index
                .entry(key.to_string())
                .or_default()
                .push(room_id.to_string());
        }
    }

    pub fn add_connection(&mut self, from_id: &str, to_id: &str, relation: &str) {
        let strength = 0.5;
        self.connections.push(Connection {
            from: from_id.to_string(),
            to: to_id.to_string(),
            strength,
            relation: relation.to_string(),
        });
    }

    pub fn recall(&mut self, key: &str) -> Vec<(&MemoryItem, &Room)> {
        let mut results = Vec::new();

        if let Some(room_ids) = self.index.get(key).cloned() {
            for room_id in &room_ids {
                if let Some(room_idx) = self.rooms.iter().position(|r| &r.id == room_id) {
                    let room = &self.rooms[room_idx];
                    for item in &room.items {
                        if item.key == key {
                            let item_ptr = item as *const MemoryItem;
                            let room_ptr = room as *const Room;
                            unsafe {
                                results.push((&*item_ptr, &*room_ptr));
                            }
                        }
                    }
                    self.rooms[room_idx].items.iter_mut()
                        .filter(|i| i.key == key)
                        .for_each(|i| i.access_count += 1);
                }
            }
        }

        results.sort_by(|a, b| b.0.confidence.partial_cmp(&a.0.confidence).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn strengthen(&mut self, room_id: &str, amount: f32) {
        if let Some(room) = self.rooms.iter_mut().find(|r| r.id == room_id) {
            room.importance = (room.importance + amount).min(1.0);
        }
    }

    pub fn prune(&mut self, threshold: f32) {
        self.rooms.retain(|r| r.importance >= threshold);
        self.connections.retain(|c| {
            self.rooms.iter().any(|r| r.id == c.from) && self.rooms.iter().any(|r| r.id == c.to)
        });
    }

    pub fn find_related(&self, room_id: &str) -> Vec<&Room> {
        let connected_ids: Vec<&str> = self.connections
            .iter()
            .filter(|c| c.from == room_id || c.to == room_id)
            .map(|c| if c.from == room_id { c.to.as_str() } else { c.from.as_str() })
            .collect();

        self.rooms.iter()
            .filter(|r| connected_ids.contains(&r.id.as_str()))
            .collect()
    }

    pub fn summary(&self) -> PalaceSummary {
        let total_items: usize = self.rooms.iter().map(|r| r.items.len()).sum();
        let avg_importance = if self.rooms.is_empty() {
            0.0
        } else {
            self.rooms.iter().map(|r| r.importance).sum::<f32>() / self.rooms.len() as f32
        };

        PalaceSummary {
            total_rooms: self.rooms.len(),
            total_items,
            total_connections: self.connections.len(),
            avg_importance,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalaceSummary {
    pub total_rooms: usize,
    pub total_items: usize,
    pub total_connections: usize,
    pub avg_importance: f32,
}

#[cfg(test)]
mod memory_tests {
    use crate::memory::{MemoryPalace, RoomType};

    #[test]
    fn test_palace_creation() {
        let palace = MemoryPalace::new();
        assert!(palace.rooms.is_empty());
        assert!(palace.connections.is_empty());
    }

    #[test]
    fn test_add_room() {
        let mut palace = MemoryPalace::new();
        let id = palace.add_room("Concepts", RoomType::Concept);

        assert_eq!(id, "room-1");
        assert_eq!(palace.rooms.len(), 1);
        assert_eq!(palace.rooms[0].name, "Concepts");
    }

    #[test]
    fn test_add_item() {
        let mut palace = MemoryPalace::new();
        let room_id = palace.add_room("Patterns", RoomType::Pattern);

        palace.add_item(&room_id, "builder", "Use builder pattern for complex structs", 0.9);

        assert_eq!(palace.rooms[0].items.len(), 1);
        assert!(palace.index.contains_key("builder"));
    }

    #[test]
    fn test_recall() {
        let mut palace = MemoryPalace::new();
        let room_id = palace.add_room("Patterns", RoomType::Pattern);
        palace.add_item(&room_id, "builder", "Builder pattern", 0.9);

        let results = palace.recall("builder");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.key, "builder");
    }

    #[test]
    fn test_recall_nonexistent() {
        let mut palace = MemoryPalace::new();
        let results = palace.recall("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_add_connection() {
        let mut palace = MemoryPalace::new();
        let r1 = palace.add_room("A", RoomType::Concept);
        let r2 = palace.add_room("B", RoomType::Pattern);

        palace.add_connection(&r1, &r2, "related_to");

        assert_eq!(palace.connections.len(), 1);
        assert_eq!(palace.connections[0].relation, "related_to");
    }

    #[test]
    fn test_find_related() {
        let mut palace = MemoryPalace::new();
        let r1 = palace.add_room("A", RoomType::Concept);
        let r2 = palace.add_room("B", RoomType::Pattern);
        let r3 = palace.add_room("C", RoomType::Decision);

        palace.add_connection(&r1, &r2, "uses");
        palace.add_connection(&r1, &r3, "depends");

        let related = palace.find_related(&r1);
        assert_eq!(related.len(), 2);
    }

    #[test]
    fn test_strengthen() {
        let mut palace = MemoryPalace::new();
        let room_id = palace.add_room("Test", RoomType::Learning);

        palace.strengthen(&room_id, 0.3);
        assert!((palace.rooms[0].importance - 0.8).abs() < 0.01);

        palace.strengthen(&room_id, 0.5);
        assert!((palace.rooms[0].importance - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_prune() {
        let mut palace = MemoryPalace::new();
        let r1 = palace.add_room("Keep", RoomType::Concept);
        let _r2 = palace.add_room("Remove", RoomType::Concept);

        palace.strengthen(&r1, 0.5);
        palace.prune(0.6);

        assert_eq!(palace.rooms.len(), 1);
        assert_eq!(palace.rooms[0].name, "Keep");
    }

    #[test]
    fn test_summary() {
        let mut palace = MemoryPalace::new();
        let r1 = palace.add_room("A", RoomType::Concept);
        palace.add_item(&r1, "key1", "value1", 0.9);
        palace.add_item(&r1, "key2", "value2", 0.8);

        let summary = palace.summary();
        assert_eq!(summary.total_rooms, 1);
        assert_eq!(summary.total_items, 2);
    }
}

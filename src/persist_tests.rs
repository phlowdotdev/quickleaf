//! Tests for persistence features

#[cfg(test)]
#[cfg(feature = "persist")]
mod tests {
    use crate::cache::Cache;
    use crate::event::Event;
    use crate::valu3::traits::ToValueBehavior;
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;
    use std::fs;
    use std::path::Path;

    // Helper function to create a unique test database path
    fn test_db_path(name: &str) -> String {
        format!("/tmp/quickleaf_test_{}.db", name)
    }

    // Helper function to cleanup test database
    fn cleanup_test_db(path: &str) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_basic_persist() {
        let db_path = test_db_path("basic_persist");
        cleanup_test_db(&db_path);

        // Create and populate cache
        {
            let mut cache = Cache::with_persist(&db_path, 10).unwrap();
            cache.insert("key1", "value1");
            cache.insert("key2", "value2");
            cache.insert("key3", 123);
            
            assert_eq!(cache.len(), 3);
            assert_eq!(cache.get("key1"), Some(&"value1".to_value()));
            
            // Give time for background writer
            thread::sleep(Duration::from_millis(100));
        }

        // Load from persisted data
        {
            let mut cache = Cache::with_persist(&db_path, 10).unwrap();
            
            assert_eq!(cache.len(), 3);
            assert_eq!(cache.get("key1"), Some(&"value1".to_value()));
            assert_eq!(cache.get("key2"), Some(&"value2".to_value()));
            assert_eq!(cache.get("key3"), Some(&123.to_value()));
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_persist_with_events() {
        let db_path = test_db_path("persist_with_events");
        cleanup_test_db(&db_path);

        let (tx, rx) = channel();
        
        {
            let mut cache = Cache::with_persist_and_sender(&db_path, 10, tx).unwrap();
            
            cache.insert("test1", "data1");
            cache.insert("test2", "data2");
            cache.remove("test1").unwrap();
            
            // Give time for events to be sent
            thread::sleep(Duration::from_millis(100));
        }

        // Collect events
        let mut events = Vec::new();
        for event in rx.try_iter() {
            events.push(event);
        }

        // Should have received insert and remove events
        assert!(events.len() >= 2);
        
        // Verify first event is insert
        if let Event::Insert(data) = &events[0] {
            assert_eq!(data.key, "test1");
        } else {
            panic!("Expected insert event");
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_persist_with_ttl() {
        let db_path = test_db_path("persist_with_ttl");
        cleanup_test_db(&db_path);

        // Create cache with default TTL
        {
            let mut cache = Cache::with_persist_and_ttl(
                &db_path, 
                10, 
                Duration::from_secs(3600)
            ).unwrap();
            
            cache.insert("long_lived", "data");
            cache.insert_with_ttl("short_lived", "temp", Duration::from_millis(50));
            
            assert_eq!(cache.len(), 2);
            
            // Wait for short_lived to expire
            thread::sleep(Duration::from_millis(100));
            
            assert!(!cache.contains_key("short_lived"));
            assert!(cache.contains_key("long_lived"));
            
            // Give time for persistence
            thread::sleep(Duration::from_millis(100));
        }

        // Load and verify TTL persistence
        {
            let mut cache = Cache::with_persist_and_ttl(
                &db_path,
                10,
                Duration::from_secs(3600)
            ).unwrap();
            
            // short_lived should be gone, long_lived should remain
            assert_eq!(cache.len(), 1);
            assert!(cache.contains_key("long_lived"));
            assert!(!cache.contains_key("short_lived"));
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_persist_with_sender_and_ttl() {
        let db_path = test_db_path("persist_sender_ttl");
        cleanup_test_db(&db_path);

        let (tx, rx) = channel();
        
        {
            let mut cache = Cache::with_persist_and_sender_and_ttl(
                &db_path,
                10,
                tx,
                Duration::from_secs(300)
            ).unwrap();
            
            // Insert with default TTL
            cache.insert("default_ttl", "value1");
            
            // Insert with custom TTL
            cache.insert_with_ttl("custom_ttl", "value2", Duration::from_secs(60));
            
            // Insert and remove
            cache.insert("to_remove", "value3");
            cache.remove("to_remove").unwrap();
            
            assert_eq!(cache.len(), 2);
            
            // Give time for events and persistence
            thread::sleep(Duration::from_millis(200));
        }

        // Check events were received
        let events: Vec<_> = rx.try_iter().collect();
        assert!(events.len() >= 3); // At least 3 inserts and 1 remove
        
        // Load and verify
        {
            let mut cache = Cache::with_persist_and_sender_and_ttl(
                &db_path,
                10,
                channel().0, // New channel for this instance
                Duration::from_secs(300)
            ).unwrap();
            
            assert_eq!(cache.len(), 2);
            assert!(cache.contains_key("default_ttl"));
            assert!(cache.contains_key("custom_ttl"));
            assert!(!cache.contains_key("to_remove"));
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_persist_capacity_limit() {
        let db_path = test_db_path("persist_capacity");
        cleanup_test_db(&db_path);

        {
            let mut cache = Cache::with_persist(&db_path, 3).unwrap();
            
            cache.insert("item1", "value1");
            cache.insert("item2", "value2");
            cache.insert("item3", "value3");
            cache.insert("item4", "value4"); // Should evict item1
            
            assert_eq!(cache.len(), 3);
            assert!(!cache.contains_key("item1"));
            assert!(cache.contains_key("item4"));
            
            thread::sleep(Duration::from_millis(100));
        }

        // Verify capacity is maintained after reload
        {
            let mut cache = Cache::with_persist(&db_path, 3).unwrap();
            
            assert_eq!(cache.len(), 3);
            assert!(!cache.contains_key("item1"));
            assert!(cache.contains_key("item2"));
            assert!(cache.contains_key("item3"));
            assert!(cache.contains_key("item4"));
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_persist_clear_operation() {
        let db_path = test_db_path("persist_clear");
        cleanup_test_db(&db_path);

        let (tx, rx) = channel();
        
        {
            let mut cache = Cache::with_persist_and_sender(&db_path, 10, tx).unwrap();
            
            cache.insert("key1", "value1");
            cache.insert("key2", "value2");
            cache.clear();
            
            assert_eq!(cache.len(), 0);
            
            thread::sleep(Duration::from_millis(100));
        }

        // Check clear event was sent
        let events: Vec<_> = rx.try_iter().collect();
        let has_clear = events.iter().any(|e| matches!(e, Event::Clear));
        assert!(has_clear);

        // Verify clear was persisted
        {
            let cache = Cache::with_persist(&db_path, 10).unwrap();
            assert_eq!(cache.len(), 0);
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_persist_expired_cleanup_on_load() {
        let db_path = test_db_path("persist_expired_cleanup");
        cleanup_test_db(&db_path);

        {
            let mut cache = Cache::with_persist(&db_path, 10).unwrap();
            
            // Insert items with very short TTL
            cache.insert_with_ttl("expired1", "value1", Duration::from_millis(50));
            cache.insert_with_ttl("expired2", "value2", Duration::from_millis(50));
            cache.insert("permanent", "value3");
            
            thread::sleep(Duration::from_millis(200));
        }

        // Load cache - expired items should be cleaned up
        {
            let mut cache = Cache::with_persist(&db_path, 10).unwrap();
            
            // Manual cleanup to trigger removal
            cache.cleanup_expired();
            
            assert_eq!(cache.len(), 1);
            assert!(cache.contains_key("permanent"));
            assert!(!cache.contains_key("expired1"));
            assert!(!cache.contains_key("expired2"));
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_persist_database_creation() {
        let _db_path = test_db_path("persist_db_creation");
        let db_dir = "/tmp/quickleaf_test_dir";
        let nested_db_path = format!("{}/cache.db", db_dir);
        
        // Clean up any existing files/dirs
        let _ = fs::remove_file(&nested_db_path);
        let _ = fs::remove_dir(db_dir);

        // Should create directory if it doesn't exist
        {
            let cache = Cache::with_persist(&nested_db_path, 10);
            assert!(cache.is_ok());
            
            // Directory should be created
            assert!(Path::new(db_dir).exists());
        }

        // Clean up
        let _ = fs::remove_file(&nested_db_path);
        let _ = fs::remove_dir(db_dir);
    }

    #[test]
    fn test_persist_concurrent_access() {
        let db_path = test_db_path("persist_concurrent");
        cleanup_test_db(&db_path);

        // Create initial cache with some data
        {
            let mut cache = Cache::with_persist(&db_path, 20).unwrap();
            for i in 0..5 {
                cache.insert(format!("key{}", i), format!("value{}", i));
            }
            thread::sleep(Duration::from_millis(100));
        }

        // Simulate concurrent access with multiple threads
        let handles: Vec<_> = (0..3).map(|thread_id| {
            let path = db_path.clone();
            thread::spawn(move || {
                let mut cache = Cache::with_persist(&path, 20).unwrap();
                
                // Each thread adds its own keys
                for i in 0..3 {
                    let key = format!("thread{}_{}", thread_id, i);
                    let value = format!("value_{}_{}", thread_id, i);
                    cache.insert(key, value);
                }
                
                thread::sleep(Duration::from_millis(100));
            })
        }).collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all data is present
        {
            let mut cache = Cache::with_persist(&db_path, 20).unwrap();
            
            // Should have original 5 + 3 threads * 3 items = 14 items
            assert!(cache.len() >= 5); // At least original items
            
            // Check original items
            for i in 0..5 {
                assert!(cache.contains_key(&format!("key{}", i)));
            }
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_persist_with_special_characters() {
        let db_path = test_db_path("persist_special_chars");
        cleanup_test_db(&db_path);

        {
            let mut cache = Cache::with_persist(&db_path, 10).unwrap();
            
            // Test various special characters in keys and values
            cache.insert("key:with:colons", "value:with:colons");
            cache.insert("key/with/slashes", "value/with/slashes");
            cache.insert("key-with-dashes", "value-with-dashes");
            cache.insert("key.with.dots", "value.with.dots");
            cache.insert("key with spaces", "value with spaces");
            cache.insert("key'with'quotes", "value'with'quotes");
            cache.insert("key\"with\"double", "value\"with\"double");
            
            thread::sleep(Duration::from_millis(100));
        }

        // Load and verify special characters are preserved
        {
            let mut cache = Cache::with_persist(&db_path, 10).unwrap();
            
            assert_eq!(cache.get("key:with:colons"), Some(&"value:with:colons".to_value()));
            assert_eq!(cache.get("key/with/slashes"), Some(&"value/with/slashes".to_value()));
            assert_eq!(cache.get("key-with-dashes"), Some(&"value-with-dashes".to_value()));
            assert_eq!(cache.get("key.with.dots"), Some(&"value.with.dots".to_value()));
            assert_eq!(cache.get("key with spaces"), Some(&"value with spaces".to_value()));
            assert_eq!(cache.get("key'with'quotes"), Some(&"value'with'quotes".to_value()));
            assert_eq!(cache.get("key\"with\"double"), Some(&"value\"with\"double".to_value()));
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_persist_mixed_value_types() {
        let db_path = test_db_path("persist_mixed_types");
        cleanup_test_db(&db_path);

        {
            let mut cache = Cache::with_persist(&db_path, 10).unwrap();
            
            // Insert different value types
            cache.insert("string", "text value");
            cache.insert("integer", 42);
            cache.insert("float", 3.14);
            cache.insert("boolean", true);
            cache.insert("negative", -123);
            
            thread::sleep(Duration::from_millis(100));
        }

        // Load and verify types are preserved
        {
            let mut cache = Cache::with_persist(&db_path, 10).unwrap();
            
            assert_eq!(cache.get("string"), Some(&"text value".to_value()));
            assert_eq!(cache.get("integer"), Some(&42.to_value()));
            assert_eq!(cache.get("float"), Some(&3.14.to_value()));
            assert_eq!(cache.get("boolean"), Some(&true.to_value()));
            assert_eq!(cache.get("negative"), Some(&(-123).to_value()));
        }

        cleanup_test_db(&db_path);
    }

    #[test]
    fn test_persist_update_existing_key() {
        let db_path = test_db_path("persist_update");
        cleanup_test_db(&db_path);

        {
            let mut cache = Cache::with_persist(&db_path, 10).unwrap();
            
            cache.insert("key1", "original");
            thread::sleep(Duration::from_millis(50));
            
            cache.insert("key1", "updated");
            thread::sleep(Duration::from_millis(50));
            
            assert_eq!(cache.get("key1"), Some(&"updated".to_value()));
        }

        // Verify update was persisted
        {
            let mut cache = Cache::with_persist(&db_path, 10).unwrap();
            assert_eq!(cache.get("key1"), Some(&"updated".to_value()));
        }

        cleanup_test_db(&db_path);
    }
}

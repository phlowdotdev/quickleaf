#[cfg(test)]
mod ttl_tests {
    use crate::{Cache, CacheItem};
    use std::thread;
    use std::time::Duration;
    use valu3::traits::ToValueBehavior;

    #[test]
    fn test_cache_item_creation() {
        let item = CacheItem::new(42.to_value());
        assert_eq!(item.value, 42.to_value());
        assert!(item.ttl().is_none());
        assert!(!item.is_expired());
    }

    #[test]
    fn test_cache_item_with_ttl() {
        let ttl = Duration::from_millis(100);
        let item = CacheItem::with_ttl(42.to_value(), ttl);
        assert_eq!(item.value, 42.to_value());
        assert_eq!(item.ttl(), Some(ttl));
        assert!(!item.is_expired());

        thread::sleep(Duration::from_millis(150));
        assert!(item.is_expired());
    }

    #[test]
    fn test_cache_with_default_ttl() {
        let ttl = Duration::from_secs(300);
        let mut cache = Cache::with_default_ttl(10, ttl);

        assert_eq!(cache.get_default_ttl(), Some(ttl));

        cache.insert("test", 42);
        assert_eq!(cache.get("test"), Some(&42.to_value()));
    }

    #[test]
    fn test_cache_insert_with_ttl() {
        let mut cache = Cache::new(10);
        let ttl = Duration::from_millis(100);

        cache.insert_with_ttl("test", 42, ttl);
        assert_eq!(cache.get("test"), Some(&42.to_value()));

        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get("test"), None);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_lazy_cleanup_on_get() {
        let mut cache = Cache::new(10);
        let ttl = Duration::from_millis(50);

        cache.insert_with_ttl("expired", 1, ttl);
        cache.insert("normal", 2);

        assert_eq!(cache.len(), 2);

        thread::sleep(Duration::from_millis(100));

        assert_eq!(cache.get("expired"), None);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get("normal"), Some(&2.to_value()));
    }

    #[test]
    fn test_cleanup_expired() {
        let mut cache = Cache::new(10);
        let ttl = Duration::from_millis(50);

        cache.insert_with_ttl("expired1", 1, ttl);
        cache.insert_with_ttl("expired2", 2, ttl);
        cache.insert("normal", 3);

        assert_eq!(cache.len(), 3);

        thread::sleep(Duration::from_millis(100));

        let removed_count = cache.cleanup_expired();
        assert_eq!(removed_count, 2);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get("normal"), Some(&3.to_value()));
    }

    #[test]
    fn test_contains_key_with_expired() {
        let mut cache = Cache::new(10);
        let ttl = Duration::from_millis(50);

        cache.insert_with_ttl("test", 42, ttl);
        assert!(cache.contains_key("test"));

        thread::sleep(Duration::from_millis(100));
        assert!(!cache.contains_key("test"));
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_list_filters_expired_items() {
        let mut cache = Cache::new(10);
        let ttl = Duration::from_millis(50);

        cache.insert_with_ttl("expired", 1, ttl);
        cache.insert("normal1", 2);
        cache.insert("normal2", 3);

        assert_eq!(cache.len(), 3);

        thread::sleep(Duration::from_millis(100));

        let result = cache.list(crate::ListProps::default()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_set_default_ttl() {
        let mut cache = Cache::new(10);
        assert_eq!(cache.get_default_ttl(), None);

        let ttl = Duration::from_secs(60);
        cache.set_default_ttl(Some(ttl));
        assert_eq!(cache.get_default_ttl(), Some(ttl));

        cache.set_default_ttl(None);
        assert_eq!(cache.get_default_ttl(), None);
    }
}

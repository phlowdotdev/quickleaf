use indexmap::IndexMap;
use std::fmt::Debug;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use valu3::traits::ToValueBehavior;
use valu3::value::Value;

use crate::error::Error;
use crate::event::Event;
use crate::filter::Filter;
use crate::list_props::{ListProps, Order, StartAfter};
use std::sync::mpsc::Sender;

#[cfg(feature = "persist")]
use std::path::Path;
#[cfg(feature = "persist")]
use std::sync::mpsc::channel;

/// Type alias for cache keys.
pub type Key = String;

/// Helper function to get current time in milliseconds since UNIX_EPOCH
#[inline(always)]
fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

/// Optimized cache item with integer timestamps for faster TTL checks
#[derive(Clone, Debug)]
pub struct CacheItem {
    /// The stored value
    pub value: Value,
    /// When this item was created (millis since epoch)
    pub created_at: u64,
    /// Optional TTL in milliseconds
    pub ttl_millis: Option<u64>,
}

impl CacheItem {
    #[inline]
    pub fn new(value: Value) -> Self {
        Self {
            value,
            created_at: current_time_millis(),
            ttl_millis: None,
        }
    }

    #[inline]
    pub fn with_ttl(value: Value, ttl: Duration) -> Self {
        Self {
            value,
            created_at: current_time_millis(),
            ttl_millis: Some(ttl.as_millis() as u64),
        }
    }

    #[inline(always)]
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_millis {
            (current_time_millis() - self.created_at) > ttl
        } else {
            false
        }
    }
    
    /// Convert back to SystemTime for compatibility
    pub fn created_at_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(self.created_at)
    }
    
    /// Get TTL as Duration for compatibility
    pub fn ttl(&self) -> Option<Duration> {
        self.ttl_millis.map(Duration::from_millis)
    }
}

impl PartialEq for CacheItem {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.ttl_millis == other.ttl_millis
    }
}

/// Optimized cache using IndexMap for O(1) operations with maintained insertion order
#[derive(Clone, Debug)]
pub struct Cache {
    // IndexMap maintains insertion order and provides O(1) for all operations
    map: IndexMap<Key, CacheItem>,
    capacity: usize,
    default_ttl: Option<Duration>,
    sender: Option<Sender<Event>>,
    #[cfg(feature = "persist")]
    persist_path: Option<std::path::PathBuf>,
}

impl PartialEq for Cache {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
            && self.capacity == other.capacity
            && self.default_ttl == other.default_ttl
    }
}

impl Cache {
    /// Creates a new cache with the specified capacity
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self {
            map: IndexMap::with_capacity(capacity),
            capacity,
            default_ttl: None,
            sender: None,
            #[cfg(feature = "persist")]
            persist_path: None,
        }
    }

    /// Creates a new cache with event notifications
    pub fn with_sender(capacity: usize, sender: Sender<Event>) -> Self {
        Self {
            map: IndexMap::with_capacity(capacity),
            capacity,
            default_ttl: None,
            sender: Some(sender),
            #[cfg(feature = "persist")]
            persist_path: None,
        }
    }

    /// Creates a new cache with default TTL for all items
    pub fn with_default_ttl(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            map: IndexMap::with_capacity(capacity),
            capacity,
            default_ttl: Some(default_ttl),
            sender: None,
            #[cfg(feature = "persist")]
            persist_path: None,
        }
    }

    /// Creates a new cache with both event notifications and default TTL
    pub fn with_sender_and_ttl(
        capacity: usize,
        sender: Sender<Event>,
        default_ttl: Duration,
    ) -> Self {
        Self {
            map: IndexMap::with_capacity(capacity),
            capacity,
            default_ttl: Some(default_ttl),
            sender: Some(sender),
            #[cfg(feature = "persist")]
            persist_path: None,
        }
    }

    #[cfg(feature = "persist")]
    pub fn with_persist<P: AsRef<Path>>(
        path: P,
        capacity: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::sqlite_store::{ensure_db_file, items_from_db, spawn_writer, PersistentEvent};
        
        let path = path.as_ref().to_path_buf();
        ensure_db_file(&path)?;
        
        let (event_tx, event_rx) = channel();
        let (persist_tx, persist_rx) = channel();
        
        spawn_writer(path.clone(), persist_rx);
        
        let mut cache = Self::with_sender(capacity, event_tx);
        cache.persist_path = Some(path.clone());
        
        std::thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                let persistent_event = PersistentEvent::new(event.clone());
                if persist_tx.send(persistent_event).is_err() {
                    break;
                }
            }
        });
        
        // Load existing data - convert old format to new
        let items = items_from_db(&path)?;
        for (key, old_item) in items {
            if cache.map.len() < capacity {
                // Convert old CacheItem to new format
                let new_item = CacheItem {
                    value: old_item.value,
                    created_at: old_item.created_at
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or(Duration::ZERO)
                        .as_millis() as u64,
                    ttl_millis: old_item.ttl.map(|d| d.as_millis() as u64),
                };
                cache.map.insert(key, new_item);
            }
        }
        
        // Sort by keys to maintain order
        cache.map.sort_keys();
        
        Ok(cache)
    }

    pub fn set_event(&mut self, sender: Sender<Event>) {
        self.sender = Some(sender);
    }

    pub fn remove_event(&mut self) {
        self.sender = None;
    }

    #[inline]
    fn send_insert(&self, key: Key, value: Value) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(Event::insert(key, value));
        }
    }

    #[inline]
    fn send_remove(&self, key: Key, value: Value) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(Event::remove(key, value));
        }
    }

    #[inline]
    fn send_clear(&self) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(Event::clear());
        }
    }

    /// Optimized insert with IndexMap - maintains sorted order automatically
    pub fn insert<T, V>(&mut self, key: T, value: V)
    where
        T: Into<String> + Clone + AsRef<str>,
        V: ToValueBehavior,
    {
        let key = key.into();
        let item = if let Some(default_ttl) = self.default_ttl {
            CacheItem::with_ttl(value.to_value(), default_ttl)
        } else {
            CacheItem::new(value.to_value())
        };

        // Check if value is the same
        if let Some(existing_item) = self.map.get(&key) {
            if existing_item.value == item.value {
                return;
            }
        }

        // LRU eviction if at capacity
        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            // Remove first (oldest) item - O(1) with IndexMap!
            if let Some((removed_key, removed_item)) = self.map.shift_remove_index(0) {
                self.send_remove(removed_key, removed_item.value);
            }
        }

        // Insert and sort to maintain order
        let old = self.map.insert(key.clone(), item.clone());
        
        // Sort keys to maintain alphabetical order
        self.map.sort_keys();
        
        if old.is_none() {
            self.send_insert(key, item.value);
        }
    }

    pub fn insert_with_ttl<T, V>(&mut self, key: T, value: V, ttl: Duration)
    where
        T: Into<String> + Clone + AsRef<str>,
        V: ToValueBehavior,
    {
        let key = key.into();
        let item = CacheItem::with_ttl(value.to_value(), ttl);

        if let Some(existing_item) = self.map.get(&key) {
            if existing_item.value == item.value {
                return;
            }
        }

        // LRU eviction if at capacity
        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            if let Some((removed_key, removed_item)) = self.map.shift_remove_index(0) {
                self.send_remove(removed_key, removed_item.value);
            }
        }

        let old = self.map.insert(key.clone(), item.clone());
        self.map.sort_keys();
        
        if old.is_none() {
            self.send_insert(key.clone(), item.value.clone());
        }
        
        #[cfg(feature = "persist")]
        if let Some(persist_path) = &self.persist_path {
            if let Some(ttl_millis) = item.ttl_millis {
                let _ = crate::sqlite_store::persist_item_with_ttl(
                    persist_path,
                    &key,
                    &item.value,
                    ttl_millis / 1000,
                );
            }
        }
    }

    /// Batch insert for better performance
    pub fn insert_batch<I, T, V>(&mut self, items: I)
    where
        I: IntoIterator<Item = (T, V)>,
        T: Into<String> + Clone + AsRef<str>,
        V: ToValueBehavior,
    {
        for (key, value) in items {
            self.insert(key, value);
        }
    }

    #[inline]
    pub fn get(&mut self, key: &str) -> Option<&Value> {
        match self.map.get(key) {
            Some(item) if item.is_expired() => {
                self.remove(key).ok();
                None
            }
            Some(item) => Some(&item.value),
            None => None,
        }
    }

    pub fn get_list(&self) -> Vec<&Key> {
        self.map.keys().collect()
    }

    pub fn get_map(&self) -> IndexMap<Key, &Value> {
        self.map
            .iter()
            .filter(|(_, item)| !item.is_expired())
            .map(|(key, item)| (key.clone(), &item.value))
            .collect()
    }

    #[inline]
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        match self.map.get(key) {
            Some(item) if item.is_expired() => {
                self.remove(key).ok();
                None
            }
            _ => self.map.get_mut(key).map(|item| &mut item.value),
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
    }

    /// Optimized remove - O(1) with IndexMap!
    pub fn remove(&mut self, key: &str) -> Result<(), Error> {
        match self.map.swap_remove_entry(key) {
            Some((key, item)) => {
                self.send_remove(key, item.value);
                Ok(())
            }
            None => Err(Error::KeyNotFound),
        }
    }

    /// Batch remove for better performance
    pub fn remove_batch<'a, I>(&mut self, keys: I) -> usize
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut removed = 0;
        for key in keys {
            if self.remove(key).is_ok() {
                removed += 1;
            }
        }
        removed
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.send_clear();
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[inline]
    pub fn contains_key(&mut self, key: &str) -> bool {
        match self.map.get(key) {
            Some(item) if item.is_expired() => {
                self.remove(key).ok();
                false
            }
            Some(_) => true,
            None => false,
        }
    }

    /// Optimized cleanup using retain
    pub fn cleanup_expired(&mut self) -> usize {
        let initial_len = self.map.len();
        
        // Collect expired keys
        let expired_keys: Vec<Key> = self.map
            .iter()
            .filter(|(_, item)| item.is_expired())
            .map(|(k, _)| k.clone())
            .collect();
        
        // Remove expired items
        for key in &expired_keys {
            if let Some(item) = self.map.swap_remove(key) {
                self.send_remove(key.clone(), item.value);
            }
        }
        
        initial_len - self.map.len()
    }

    pub fn set_default_ttl(&mut self, ttl: Option<Duration>) {
        self.default_ttl = ttl;
    }

    pub fn get_default_ttl(&self) -> Option<Duration> {
        self.default_ttl
    }

    /// Optimized list with pre-allocated capacity
    pub fn list<T>(&mut self, props: T) -> Result<Vec<(Key, &Value)>, Error>
    where
        T: Into<ListProps>,
    {
        let props = props.into();
        
        // Cleanup expired first
        self.cleanup_expired();
        
        // Pre-allocate result vector
        let mut result = Vec::with_capacity(props.limit.min(self.map.len()));
        
        let iter: Box<dyn Iterator<Item = (&Key, &CacheItem)>> = match props.order {
            Order::Asc => Box::new(self.map.iter()),
            Order::Desc => Box::new(self.map.iter().rev()),
        };
        
        // Handle start_after
        let mut iter = iter;
        if let StartAfter::Key(ref start_key) = props.start_after_key {
            // Skip until we find the start key
            let mut found = false;
            for (k, _) in iter.by_ref() {
                if k == start_key {
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(Error::SortKeyNotFound);
            }
        }
        
        // Apply filters and collect results
        for (key, item) in iter {
            if item.is_expired() {
                continue;
            }
            
            let matches = match &props.filter {
                Filter::None => true,
                Filter::StartWith(prefix) => key.starts_with(prefix),
                Filter::EndWith(suffix) => key.ends_with(suffix),
                Filter::StartAndEndWith(prefix, suffix) => {
                    key.starts_with(prefix) && key.ends_with(suffix)
                }
            };
            
            if matches {
                result.push((key.clone(), &item.value));
                if result.len() >= props.limit {
                    break;
                }
            }
        }
        
        Ok(result)
    }
}

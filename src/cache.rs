use crate::error::Error;
use crate::event::Event;
use crate::filters::apply_filter_fast;
use crate::list_props::{ListProps, Order, StartAfter};
use crate::prefetch::{Prefetch, PrefetchExt};
use indexmap::IndexMap;
use std::fmt::Debug;
use std::sync::mpsc::Sender;
use std::time::{Duration, SystemTime};
use valu3::traits::ToValueBehavior;
use valu3::value::Value;

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
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

/// Represents an item stored in the cache with optional TTL (Time To Live).
///
/// Each cache item contains:
/// - The actual value stored
/// - Creation timestamp for TTL calculations
/// - Optional TTL duration for automatic expiration
///
/// # Examples
///
/// ```
/// use quickleaf::CacheItem;
/// use quickleaf::valu3::traits::ToValueBehavior;
/// use std::time::Duration;
///
/// // Create item without TTL
/// let item = CacheItem::new("Hello World".to_value());
/// assert!(!item.is_expired());
///
/// // Create item with TTL
/// let item_with_ttl = CacheItem::with_ttl("temporary".to_value(), Duration::from_secs(60));
/// assert!(!item_with_ttl.is_expired());
/// ```
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
    /// Creates a new cache item without TTL.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::CacheItem;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let item = CacheItem::new("data".to_value());
    /// assert!(!item.is_expired());
    /// assert!(item.ttl_millis.is_none());
    /// ```
    #[inline]
    pub fn new(value: Value) -> Self {
        Self {
            value,
            created_at: current_time_millis(),
            ttl_millis: None,
        }
    }

    /// Creates a new cache item with TTL.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::CacheItem;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    /// use std::time::Duration;
    ///
    /// let item = CacheItem::with_ttl("session_data".to_value(), Duration::from_secs(300));
    /// assert!(!item.is_expired());
    /// assert_eq!(item.ttl_millis, Some(300_000));
    /// ```
    #[inline]
    pub fn with_ttl(value: Value, ttl: Duration) -> Self {
        Self {
            value,
            created_at: current_time_millis(),
            ttl_millis: Some(ttl.as_millis() as u64),
        }
    }

    /// Checks if this cache item has expired based on its TTL.
    ///
    /// Returns `false` if no TTL is set (permanent item).
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::CacheItem;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    /// use std::time::Duration;
    /// use std::thread;
    ///
    /// // Item without TTL never expires
    /// let permanent_item = CacheItem::new("permanent".to_value());
    /// assert!(!permanent_item.is_expired());
    ///
    /// // Item with very short TTL
    /// let short_lived = CacheItem::with_ttl("temp".to_value(), Duration::from_millis(1));
    /// thread::sleep(Duration::from_millis(10));
    /// assert!(short_lived.is_expired());
    /// ```
    #[inline(always)]
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_millis {
            (current_time_millis() - self.created_at) > ttl
        } else {
            false
        }
    }

    /// Get TTL as Duration for compatibility
    #[inline]
    pub fn ttl(&self) -> Option<Duration> {
        self.ttl_millis.map(Duration::from_millis)
    }

    /// Convert back to SystemTime for compatibility  
    #[inline]
    pub fn created_at_time(&self) -> SystemTime {
        std::time::UNIX_EPOCH + Duration::from_millis(self.created_at)
    }
}

impl PartialEq for CacheItem {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.ttl_millis == other.ttl_millis
    }
}

/// Core cache implementation with LRU eviction, TTL support, and event notifications.
///
/// This cache provides:
/// - O(1) access time for get/insert operations
/// - LRU (Least Recently Used) eviction when capacity is reached
/// - Optional TTL (Time To Live) for automatic expiration
/// - Event notifications for cache operations
/// - Filtering and ordering capabilities for listing entries
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use quickleaf::Cache;
/// use quickleaf::valu3::traits::ToValueBehavior;
///
/// let mut cache = Cache::new(3);
/// cache.insert("key1", "value1");
/// cache.insert("key2", "value2");
///
/// assert_eq!(cache.get("key1"), Some(&"value1".to_value()));
/// assert_eq!(cache.len(), 2);
/// ```
///
/// ## With TTL Support
///
/// ```
/// use quickleaf::Cache;
/// use quickleaf::valu3::traits::ToValueBehavior;
/// use std::time::Duration;
///
/// let mut cache = Cache::with_default_ttl(10, Duration::from_secs(60));
/// cache.insert("session", "user_data");  
/// cache.insert_with_ttl("temp", "data", Duration::from_millis(100));  
///
/// assert!(cache.contains_key("session"));
/// ```
///
/// ## With Event Notifications
///
/// ```
/// use quickleaf::Cache;
/// use quickleaf::Event;
/// use quickleaf::valu3::traits::ToValueBehavior;
/// use std::sync::mpsc::channel;
///
/// let (tx, rx) = channel();
/// let mut cache = Cache::with_sender(5, tx);
///
/// cache.insert("notify", "me");
///
/// // Receive the insert event
/// if let Ok(event) = rx.try_recv() {
///     match event {
///         Event::Insert(data) => {
///             assert_eq!(data.key, "notify");
///             assert_eq!(data.value, "me".to_value());
///         },
///         _ => panic!("Expected insert event"),
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Cache {
    map: IndexMap<Key, CacheItem>,
    capacity: usize,
    default_ttl: Option<Duration>,
    sender: Option<Sender<Event>>,
    #[cfg(feature = "persist")]
    persist_path: Option<std::path::PathBuf>,
    _phantom: std::marker::PhantomData<Value>,
}

impl PartialEq for Cache {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
            && self.capacity == other.capacity
            && self.default_ttl == other.default_ttl
    }
}

impl Cache {
    /// Creates a new cache with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Cache;
    ///
    /// let cache = Cache::new(100);
    /// assert_eq!(cache.capacity(), 100);
    /// assert!(cache.is_empty());
    /// ```
    pub fn new(capacity: usize) -> Self {
        Self {
            map: IndexMap::with_capacity(capacity),
            capacity,
            default_ttl: None,
            sender: None,
            #[cfg(feature = "persist")]
            persist_path: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Creates a new cache with event notifications.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Cache;
    /// use quickleaf::Event;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    /// use std::sync::mpsc::channel;
    ///
    /// let (tx, rx) = channel();
    /// let mut cache = Cache::with_sender(10, tx);
    ///
    /// cache.insert("test", 42);
    ///
    /// // Event should be received
    /// assert!(rx.try_recv().is_ok());
    /// ```
    pub fn with_sender(capacity: usize, sender: Sender<Event>) -> Self {
        Self {
            map: IndexMap::with_capacity(capacity),
            capacity,
            default_ttl: None,
            sender: Some(sender),
            #[cfg(feature = "persist")]
            persist_path: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Creates a new cache with default TTL for all items.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Cache;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    /// use std::time::Duration;
    ///
    /// let mut cache = Cache::with_default_ttl(10, Duration::from_secs(300));
    /// cache.insert("auto_expire", "data");
    ///
    /// assert_eq!(cache.get_default_ttl(), Some(Duration::from_secs(300)));
    /// ```
    pub fn with_default_ttl(capacity: usize, default_ttl: Duration) -> Self {
        Self {
            map: IndexMap::with_capacity(capacity),
            capacity,
            default_ttl: Some(default_ttl),
            sender: None,
            #[cfg(feature = "persist")]
            persist_path: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Creates a new cache with both event notifications and default TTL.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Cache;
    /// use quickleaf::Event;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    /// use std::sync::mpsc::channel;
    /// use std::time::Duration;
    ///
    /// let (tx, rx) = channel();
    /// let mut cache = Cache::with_sender_and_ttl(10, tx, Duration::from_secs(60));
    ///
    /// cache.insert("monitored", "data");
    /// assert!(rx.try_recv().is_ok());
    /// assert_eq!(cache.get_default_ttl(), Some(Duration::from_secs(60)));
    /// ```
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
            _phantom: std::marker::PhantomData,
        }
    }

    /// Creates a new cache with SQLite persistence.
    ///
    /// This constructor enables automatic persistence of all cache operations to a SQLite database.
    /// On initialization, it will load any existing data from the database.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "persist")]
    /// # {
    /// use quickleaf::Cache;
    ///
    /// let mut cache = Cache::with_persist("data/cache.db", 1000).unwrap();
    /// cache.insert("persistent_key", "persistent_value");
    /// # }
    /// ```
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

        let mut items = items_from_db(&path)?;

        items.sort_by(|a, b| a.0.cmp(&b.0));

        for (key, item) in items {
            if cache.map.len() < capacity {
                cache.map.insert(key, item);
            }
        }

        Ok(cache)
    }

    /// Creates a new cache with SQLite persistence and event notifications.
    ///
    /// This constructor combines SQLite persistence with custom event notifications.
    /// You'll receive events for cache operations while data is also persisted to SQLite.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SQLite database file
    /// * `capacity` - Maximum number of items the cache can hold
    /// * `sender` - Channel sender for event notifications
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "persist")]
    /// # {
    /// use quickleaf::Cache;
    /// use std::sync::mpsc::channel;
    ///
    /// let (tx, rx) = channel();
    /// let mut cache = Cache::with_persist_and_sender("data/cache.db", 1000, tx).unwrap();
    ///
    /// cache.insert("key", "value");
    ///
    /// // Receive events for persisted operations
    /// for event in rx.try_iter() {
    ///     println!("Event: {:?}", event);
    /// }
    /// # }
    /// ```
    #[cfg(feature = "persist")]
    pub fn with_persist_and_sender<P: AsRef<Path>>(
        path: P,
        capacity: usize,
        external_sender: Sender<Event>,
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
                let _ = external_sender.send(event.clone());

                let persistent_event = PersistentEvent::new(event);
                if persist_tx.send(persistent_event).is_err() {
                    break;
                }
            }
        });

        let mut items = items_from_db(&path)?;

        items.sort_by(|a, b| a.0.cmp(&b.0));

        for (key, item) in items {
            if cache.map.len() < capacity {
                cache.map.insert(key, item);
            }
        }

        Ok(cache)
    }

    /// Creates a new cache with SQLite persistence and default TTL.
    ///
    /// This constructor combines SQLite persistence with a default TTL for all cache items.
    /// Items will automatically expire after the specified duration and are persisted to SQLite.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SQLite database file
    /// * `capacity` - Maximum number of items the cache can hold
    /// * `default_ttl` - Default time-to-live for all cache items
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "persist")]
    /// # {
    /// use quickleaf::Cache;
    /// use std::time::Duration;
    ///
    /// let mut cache = Cache::with_persist_and_ttl(
    ///     "data/cache.db",
    ///     1000,
    ///     Duration::from_secs(3600)
    /// ).unwrap();
    /// cache.insert("session", "data");  
    /// # }
    /// ```
    #[cfg(feature = "persist")]
    pub fn with_persist_and_ttl<P: AsRef<Path>>(
        path: P,
        capacity: usize,
        default_ttl: Duration,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::sqlite_store::{ensure_db_file, items_from_db, spawn_writer, PersistentEvent};

        let path = path.as_ref().to_path_buf();

        ensure_db_file(&path)?;

        let (event_tx, event_rx) = channel();
        let (persist_tx, persist_rx) = channel();

        spawn_writer(path.clone(), persist_rx);

        let mut cache = Self::with_sender_and_ttl(capacity, event_tx, default_ttl);
        cache.persist_path = Some(path.clone());

        std::thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                let persistent_event = PersistentEvent::new(event.clone());
                if persist_tx.send(persistent_event).is_err() {
                    break;
                }
            }
        });

        let mut items = items_from_db(&path)?;

        items.sort_by(|a, b| a.0.cmp(&b.0));

        for (key, item) in items {
            if !item.is_expired() && cache.map.len() < capacity {
                cache.map.insert(key, item);
            }
        }

        Ok(cache)
    }

    /// Creates a new cache with SQLite persistence, event notifications, and default TTL.
    ///
    /// This constructor combines all persistence features: SQLite storage, event notifications,
    /// and default TTL for all cache items. This is the most feature-complete constructor.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SQLite database file
    /// * `capacity` - Maximum number of items the cache can hold
    /// * `external_sender` - Channel sender for event notifications
    /// * `default_ttl` - Default time-to-live for all cache items
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "persist")]
    /// # {
    /// use quickleaf::Cache;
    /// use std::sync::mpsc::channel;
    /// use std::time::Duration;
    ///
    /// let (tx, rx) = channel();
    /// let mut cache = Cache::with_persist_and_sender_and_ttl(
    ///     "data/cache.db",
    ///     1000,
    ///     tx,
    ///     Duration::from_secs(3600)
    /// ).unwrap();
    ///
    /// // Insert data - it will be persisted, send events, and expire in 1 hour
    /// cache.insert("session", "user_data");
    ///
    /// // Receive events
    /// for event in rx.try_iter() {
    ///     println!("Event: {:?}", event);
    /// }
    /// # }
    /// ```
    #[cfg(feature = "persist")]
    pub fn with_persist_and_sender_and_ttl<P: AsRef<Path>>(
        path: P,
        capacity: usize,
        external_sender: Sender<Event>,
        default_ttl: Duration,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::sqlite_store::{ensure_db_file, items_from_db, spawn_writer, PersistentEvent};

        let path = path.as_ref().to_path_buf();

        ensure_db_file(&path)?;

        let (event_tx, event_rx) = channel();
        let (persist_tx, persist_rx) = channel();

        spawn_writer(path.clone(), persist_rx);

        let mut cache = Self::with_sender_and_ttl(capacity, event_tx, default_ttl);
        cache.persist_path = Some(path.clone());

        std::thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                let _ = external_sender.send(event.clone());

                let persistent_event = PersistentEvent::new(event);
                if persist_tx.send(persistent_event).is_err() {
                    break;
                }
            }
        });

        let mut items = items_from_db(&path)?;

        items.sort_by(|a, b| a.0.cmp(&b.0));

        for (key, item) in items {
            if !item.is_expired() && cache.map.len() < capacity {
                cache.map.insert(key, item);
            }
        }

        Ok(cache)
    }

    #[inline]
    pub fn set_event(&mut self, sender: Sender<Event>) {
        self.sender = Some(sender);
    }

    #[inline]
    pub fn remove_event(&mut self) {
        self.sender = None;
    }

    #[inline]
    fn send_insert(&self, key: Key, value: Value) {
        if let Some(sender) = &self.sender {
            let event = Event::insert(key, value);
            sender.send(event).unwrap();
        }
    }

    #[inline]
    fn send_remove(&self, key: Key, value: Value) {
        if let Some(sender) = &self.sender {
            let event = Event::remove(key, value);
            sender.send(event).unwrap();
        }
    }

    #[inline]
    fn send_clear(&self) {
        if let Some(sender) = &self.sender {
            let event = Event::clear();
            sender.send(event).unwrap();
        }
    }

    /// Inserts a key-value pair into the cache.
    ///
    /// If the cache is at capacity, the least recently used item will be evicted.
    /// If a default TTL is set, the item will inherit that TTL.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Cache;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let mut cache = Cache::new(2);
    /// cache.insert("key1", "value1");
    /// cache.insert("key2", "value2");
    /// cache.insert("key3", "value3");  
    ///
    /// assert_eq!(cache.get("key1"), None);  
    /// assert_eq!(cache.get("key2"), Some(&"value2".to_value()));
    /// assert_eq!(cache.get("key3"), Some(&"value3".to_value()));
    /// ```
    pub fn insert<T, V>(&mut self, key: T, value: V)
    where
        T: Into<String>,
        V: ToValueBehavior,
    {
        let key = key.into();

        let item = if let Some(default_ttl) = self.default_ttl {
            CacheItem::with_ttl(value.to_value(), default_ttl)
        } else {
            CacheItem::new(value.to_value())
        };

        if let Some(existing_item) = self.map.get(&key) {
            if existing_item.value == item.value {
                return;
            }
        }

        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            if let Some((first_key, first_item)) = self.map.shift_remove_index(0) {
                self.send_remove(first_key, first_item.value);
            }
        }

        self.map.insert(key.clone(), item.clone());

        self.send_insert(key, item.value);
    }

    /// Inserts a key-value pair with a specific TTL.
    ///
    /// The TTL overrides any default TTL set for the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Cache;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    /// use std::time::Duration;
    /// use std::thread;
    ///
    /// let mut cache = Cache::new(10);
    /// cache.insert_with_ttl("session", "user123", Duration::from_millis(100));
    ///
    /// assert!(cache.contains_key("session"));
    /// thread::sleep(Duration::from_millis(150));
    /// assert!(!cache.contains_key("session"));  
    /// ```
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

        if self.map.len() >= self.capacity && !self.map.contains_key(&key) {
            if let Some((first_key, first_item)) = self.map.shift_remove_index(0) {
                self.send_remove(first_key, first_item.value);
            }
        }

        self.map.insert(key.clone(), item.clone());

        self.send_insert(key.clone(), item.value.clone());

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

    /// Retrieves a value from the cache by key.
    ///
    /// Returns `None` if the key doesn't exist or if the item has expired.
    /// Expired items are automatically removed during this operation (lazy cleanup).
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Cache;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let mut cache = Cache::new(10);
    /// cache.insert("existing", "data");
    ///
    /// assert_eq!(cache.get("existing"), Some(&"data".to_value()));
    /// assert_eq!(cache.get("nonexistent"), None);
    /// ```
    #[inline]
    pub fn get(&mut self, key: &str) -> Option<&Value> {
        if let Some((_, item)) = self.map.get_key_value(key) {
            item.prefetch_read();
        }

        let is_expired = match self.map.get(key) {
            Some(item) => {
                if let Some(ttl) = item.ttl_millis {
                    (current_time_millis() - item.created_at) > ttl
                } else {
                    false
                }
            }
            None => return None,
        };

        if is_expired {
            if let Some(expired_item) = self.map.swap_remove(key) {
                self.send_remove(key.to_string(), expired_item.value);
            }
            None
        } else {
            self.map.get(key).map(|item| &item.value)
        }
    }

    #[inline(always)]
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

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        let should_remove = self.map.get(key).map_or(false, |item| item.is_expired());

        if should_remove {
            self.remove(key).ok();
            None
        } else {
            self.map.get_mut(key).map(|item| &mut item.value)
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
    }

    pub fn remove(&mut self, key: &str) -> Result<(), Error> {
        if let Some(item) = self.map.swap_remove(key) {
            self.send_remove(key.to_string(), item.value);
            Ok(())
        } else {
            Err(Error::KeyNotFound)
        }
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

    /// Checks if a key exists in the cache and hasn't expired.
    ///
    /// This method performs lazy cleanup of expired items.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Cache;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    /// use std::time::Duration;
    ///
    /// let mut cache = Cache::new(10);
    /// cache.insert("key", "value");
    ///
    /// assert!(cache.contains_key("key"));
    /// assert!(!cache.contains_key("nonexistent"));
    ///
    /// // Test with TTL
    /// cache.insert_with_ttl("temp", "data", Duration::from_millis(1));
    /// std::thread::sleep(Duration::from_millis(10));
    /// assert!(!cache.contains_key("temp"));  
    /// ```
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

    /// Manually removes all expired items from the cache.
    ///
    /// Returns the number of items that were removed.
    /// This is useful for proactive cleanup, though the cache also performs lazy cleanup.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Cache;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    /// use std::time::Duration;
    /// use std::thread;
    ///
    /// let mut cache = Cache::new(10);
    /// cache.insert_with_ttl("temp1", "data1", Duration::from_millis(10));
    /// cache.insert_with_ttl("temp2", "data2", Duration::from_millis(10));
    /// cache.insert("permanent", "data");
    ///
    /// thread::sleep(Duration::from_millis(20));
    ///
    /// let removed = cache.cleanup_expired();
    /// assert_eq!(removed, 2);  
    /// assert_eq!(cache.len(), 1);  
    /// ```
    pub fn cleanup_expired(&mut self) -> usize {
        let current_time = current_time_millis();
        let mut expired_keys = Vec::with_capacity(self.map.len() / 4);

        for (key, item) in &self.map {
            item.prefetch_read();

            if let Some(ttl) = item.ttl_millis {
                if (current_time - item.created_at) > ttl {
                    expired_keys.push(key.clone());
                }
            }
        }

        let removed_count = expired_keys.len();

        if !expired_keys.is_empty() {
            Prefetch::sequential_read_hints(expired_keys.as_ptr(), expired_keys.len());
        }

        for key in expired_keys {
            if let Some(item) = self.map.swap_remove(&key) {
                self.send_remove(key, item.value);
            }
        }

        removed_count
    }

    #[inline]
    pub fn set_default_ttl(&mut self, ttl: Option<Duration>) {
        self.default_ttl = ttl;
    }

    #[inline(always)]
    pub fn get_default_ttl(&self) -> Option<Duration> {
        self.default_ttl
    }

    /// Lists cache entries with filtering, ordering, and pagination support.
    ///
    /// This method automatically cleans up expired items before returning results.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Cache;
    /// use quickleaf::{ListProps, Order};
    /// use quickleaf::Filter;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let mut cache = Cache::new(10);
    /// cache.insert("apple", 1);
    /// cache.insert("banana", 2);
    /// cache.insert("apricot", 3);
    ///
    /// // List all items in ascending order
    /// let props = ListProps::default().order(Order::Asc);
    /// let items = cache.list(props).unwrap();
    /// assert_eq!(items.len(), 3);
    ///
    /// // Filter items starting with "ap"
    /// let props = ListProps::default()
    ///     .filter(Filter::StartWith("ap".to_string()));
    /// let filtered = cache.list(props).unwrap();
    /// assert_eq!(filtered.len(), 2);  
    /// ```
    pub fn list<T>(&mut self, props: T) -> Result<Vec<(Key, &Value)>, Error>
    where
        T: Into<ListProps>,
    {
        let props = props.into();

        self.cleanup_expired();

        let mut keys: Vec<String> = self.map.keys().cloned().collect();
        keys.sort();

        if !keys.is_empty() {
            Prefetch::sequential_read_hints(keys.as_ptr(), keys.len());
        }

        match props.order {
            Order::Asc => self.resolve_order(keys.iter(), props),
            Order::Desc => self.resolve_order(keys.iter().rev(), props),
        }
    }

    fn resolve_order<'a, I>(
        &self,
        mut list_iter: I,
        props: ListProps,
    ) -> Result<Vec<(Key, &Value)>, Error>
    where
        I: Iterator<Item = &'a String>,
    {
        if let StartAfter::Key(ref key) = props.start_after_key {
            list_iter
                .find(|k| k == &key)
                .ok_or(Error::SortKeyNotFound)?;
        }

        let mut list = Vec::new();
        let mut count = 0;

        for k in list_iter {
            if let Some(item) = self.map.get(k) {
                if item.is_expired() {
                    continue;
                }

                let filtered = if apply_filter_fast(k, &props.filter) {
                    Some((k.clone(), &item.value))
                } else {
                    None
                };

                if let Some(item) = filtered {
                    list.push(item);
                    count += 1;
                    if count == props.limit {
                        break;
                    }
                }
            }
        }

        Ok(list)
    }
}

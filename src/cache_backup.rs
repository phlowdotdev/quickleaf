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
    /// When this item was created
    pub created_at: SystemTime,
    /// Optional TTL duration
    pub ttl: Option<Duration>,
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
    /// assert!(item.ttl.is_none());
    /// ```
    pub fn new(value: Value) -> Self {
        Self {
            value,
            created_at: SystemTime::now(),
            ttl: None,
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
    /// assert_eq!(item.ttl, Some(Duration::from_secs(300)));
    /// ```
    pub fn with_ttl(value: Value, ttl: Duration) -> Self {
        Self {
            value,
            created_at: SystemTime::now(),
            ttl: Some(ttl),
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
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed().unwrap_or(Duration::MAX) > ttl
        } else {
            false
        }
    }
}

impl PartialEq for CacheItem {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.ttl == other.ttl
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
/// cache.insert("session", "user_data");  // Will expire in 60 seconds
/// cache.insert_with_ttl("temp", "data", Duration::from_millis(100));  // Custom TTL
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
    // Using IndexMap to maintain insertion order and get O(1) operations
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
            map: HashMap::new(),
            list: Vec::new(),
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
            map: HashMap::new(),
            list: Vec::new(),
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
            map: HashMap::new(),
            list: Vec::new(),
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
            map: HashMap::new(),
            list: Vec::new(),
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
        
        // Ensure the database file and directories exist
        ensure_db_file(&path)?;
        
        // Create channels for event handling
        let (event_tx, event_rx) = channel();
        let (persist_tx, persist_rx) = channel();
        
        // Spawn the SQLite writer thread
        spawn_writer(path.clone(), persist_rx);
        
        // Create the cache with event sender
        let mut cache = Self::with_sender(capacity, event_tx);
        cache.persist_path = Some(path.clone());
        
        // Set up event forwarding to SQLite writer
        std::thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                let persistent_event = PersistentEvent::new(event.clone());
                if persist_tx.send(persistent_event).is_err() {
                    break;
                }
            }
        });
        
        // Load existing data from database
        let items = items_from_db(&path)?;
        for (key, item) in items {
            // Directly insert into the map and list to avoid triggering events
            if cache.map.len() < capacity {
                let position = cache
                    .list
                    .iter()
                    .position(|k| k > &key)
                    .unwrap_or(cache.list.len());
                cache.list.insert(position, key.clone());
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
        
        // Ensure the database file and directories exist
        ensure_db_file(&path)?;
        
        // Create channels for internal event handling
        let (event_tx, event_rx) = channel();
        let (persist_tx, persist_rx) = channel();
        
        // Spawn the SQLite writer thread
        spawn_writer(path.clone(), persist_rx);
        
        // Create the cache with event sender
        let mut cache = Self::with_sender(capacity, event_tx);
        cache.persist_path = Some(path.clone());
        
        // Set up event forwarding to both SQLite writer and external sender
        std::thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                // Forward to external sender
                let _ = external_sender.send(event.clone());
                
                // Forward to SQLite writer
                let persistent_event = PersistentEvent::new(event);
                if persist_tx.send(persistent_event).is_err() {
                    break;
                }
            }
        });
        
        // Load existing data from database
        let items = items_from_db(&path)?;
        for (key, item) in items {
            // Directly insert into the map and list to avoid triggering events
            if cache.map.len() < capacity {
                let position = cache
                    .list
                    .iter()
                    .position(|k| k > &key)
                    .unwrap_or(cache.list.len());
                cache.list.insert(position, key.clone());
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
    /// cache.insert("session", "data");  // Will expire in 1 hour and be persisted
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
        
        // Ensure the database file and directories exist
        ensure_db_file(&path)?;
        
        // Create channels for event handling
        let (event_tx, event_rx) = channel();
        let (persist_tx, persist_rx) = channel();
        
        // Spawn the SQLite writer thread
        spawn_writer(path.clone(), persist_rx);
        
        // Create the cache with event sender and TTL
        let mut cache = Self::with_sender_and_ttl(capacity, event_tx, default_ttl);
        cache.persist_path = Some(path.clone());
        
        // Set up event forwarding to SQLite writer
        std::thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                let persistent_event = PersistentEvent::new(event.clone());
                if persist_tx.send(persistent_event).is_err() {
                    break;
                }
            }
        });
        
        // Load existing data from database
        let items = items_from_db(&path)?;
        for (key, item) in items {
            // Skip expired items during load
            if !item.is_expired() && cache.map.len() < capacity {
                let position = cache
                    .list
                    .iter()
                    .position(|k| k > &key)
                    .unwrap_or(cache.list.len());
                cache.list.insert(position, key.clone());
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
        
        // Ensure the database file and directories exist
        ensure_db_file(&path)?;
        
        // Create channels for internal event handling
        let (event_tx, event_rx) = channel();
        let (persist_tx, persist_rx) = channel();
        
        // Spawn the SQLite writer thread
        spawn_writer(path.clone(), persist_rx);
        
        // Create the cache with event sender and TTL
        let mut cache = Self::with_sender_and_ttl(capacity, event_tx, default_ttl);
        cache.persist_path = Some(path.clone());
        
        // Set up event forwarding to both SQLite writer and external sender
        std::thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                // Forward to external sender
                let _ = external_sender.send(event.clone());
                
                // Forward to SQLite writer
                let persistent_event = PersistentEvent::new(event);
                if persist_tx.send(persistent_event).is_err() {
                    break;
                }
            }
        });
        
        // Load existing data from database
        let items = items_from_db(&path)?;
        for (key, item) in items {
            // Skip expired items during load
            if !item.is_expired() && cache.map.len() < capacity {
                let position = cache
                    .list
                    .iter()
                    .position(|k| k > &key)
                    .unwrap_or(cache.list.len());
                cache.list.insert(position, key.clone());
                cache.map.insert(key, item);
            }
        }
        
        Ok(cache)
    }

    pub fn set_event(&mut self, sender: Sender<Event>) {
        self.sender = Some(sender);
    }

    pub fn remove_event(&mut self) {
        self.sender = None;
    }

    fn send_insert(&self, key: Key, value: Value) {
        if let Some(sender) = &self.sender {
            let event = Event::insert(key, value);
            sender.send(event).unwrap();
        }
    }

    fn send_remove(&self, key: Key, value: Value) {
        if let Some(sender) = &self.sender {
            let event = Event::remove(key, value);
            sender.send(event).unwrap();
        }
    }

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
    /// cache.insert("key3", "value3");  // This will evict "key1"
    ///
    /// assert_eq!(cache.get("key1"), None);  // Evicted
    /// assert_eq!(cache.get("key2"), Some(&"value2".to_value()));
    /// assert_eq!(cache.get("key3"), Some(&"value3".to_value()));
    /// ```
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

        if let Some(existing_item) = self.map.get(&key) {
            if existing_item.value == item.value {
                return;
            }
        }

        if self.map.len() != 0 && self.map.len() == self.capacity {
            let first_key = self.list.remove(0);
            let data = self.map.get(&first_key).unwrap().clone();
            self.map.remove(&first_key);
            self.send_remove(first_key, data.value);
        }

        let position = self
            .list
            .iter()
            .position(|k| k > &key)
            .unwrap_or(self.list.len());

        self.list.insert(position, key.clone());
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
    /// assert!(!cache.contains_key("session"));  // Should be expired
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

        if self.map.len() != 0 && self.map.len() == self.capacity {
            let first_key = self.list.remove(0);
            let data = self.map.get(&first_key).unwrap().clone();
            self.map.remove(&first_key);
            self.send_remove(first_key, data.value);
        }

        let position = self
            .list
            .iter()
            .position(|k| k > &key)
            .unwrap_or(self.list.len());

        self.list.insert(position, key.clone());
        self.map.insert(key.clone(), item.clone());

        self.send_insert(key.clone(), item.value.clone());
        
        // Update TTL in SQLite if we have persistence
        #[cfg(feature = "persist")]
        if let Some(persist_path) = &self.persist_path {
            if let Some(ttl_secs) = item.ttl {
                let _ = crate::sqlite_store::persist_item_with_ttl(
                    persist_path,
                    &key,
                    &item.value,
                    ttl_secs.as_secs(),
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
    pub fn get(&mut self, key: &str) -> Option<&Value> {
        // Primeiro verifica se existe e se está expirado
        let is_expired = if let Some(item) = self.map.get(key) {
            item.is_expired()
        } else {
            return None;
        };

        if is_expired {
            // Item expirado, remove do cache
            self.remove(key).ok();
            None
        } else {
            // Item válido, retorna referência
            self.map.get(key).map(|item| &item.value)
        }
    }

    pub fn get_list(&self) -> &Vec<Key> {
        &self.list
    }

    pub fn get_map(&self) -> HashMap<Key, &Value> {
        self.map
            .iter()
            .filter(|(_, item)| !item.is_expired())
            .map(|(key, item)| (key.clone(), &item.value))
            .collect()
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        // Primeiro verifica se existe e se está expirado
        let is_expired = if let Some(item) = self.map.get(key) {
            item.is_expired()
        } else {
            return None;
        };

        if is_expired {
            // Item expirado, remove do cache
            self.remove(key).ok();
            None
        } else {
            // Item válido, retorna referência mutável
            self.map.get_mut(key).map(|item| &mut item.value)
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
    }

    pub fn remove(&mut self, key: &str) -> Result<(), Error> {
        match self.list.iter().position(|k| k == &key) {
            Some(position) => {
                self.list.remove(position);

                let data = self.map.get(key).unwrap().clone();

                self.map.remove(key);

                self.send_remove(key.to_string(), data.value);

                Ok(())
            }
            None => Err(Error::KeyNotFound),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.list.clear();

        self.send_clear();
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

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
    /// assert!(!cache.contains_key("temp"));  // Should be expired and removed
    /// ```
    pub fn contains_key(&mut self, key: &str) -> bool {
        if let Some(item) = self.map.get(key) {
            if item.is_expired() {
                self.remove(key).ok();
                false
            } else {
                true
            }
        } else {
            false
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
    /// assert_eq!(removed, 2);  // temp1 and temp2 were removed
    /// assert_eq!(cache.len(), 1);  // Only permanent remains
    /// ```
    pub fn cleanup_expired(&mut self) -> usize {
        let expired_keys: Vec<_> = self
            .map
            .iter()
            .filter(|(_, item)| item.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            self.remove(&key).ok();
        }
        count
    }

    pub fn set_default_ttl(&mut self, ttl: Option<Duration>) {
        self.default_ttl = ttl;
    }

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
    /// assert_eq!(filtered.len(), 2);  // apple, apricot
    /// ```
    pub fn list<T>(&mut self, props: T) -> Result<Vec<(Key, &Value)>, Error>
    where
        T: Into<ListProps>,
    {
        let props = props.into();

        // Primeiro faz uma limpeza dos itens expirados para evitar retorná-los
        self.cleanup_expired();

        match props.order {
            Order::Asc => self.resolve_order(self.list.iter(), props),
            Order::Desc => self.resolve_order(self.list.iter().rev(), props),
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
                // Pula itens expirados (eles serão removidos na próxima limpeza)
                if item.is_expired() {
                    continue;
                }

                let filtered = match props.filter {
                    Filter::StartWith(ref key) => {
                        if k.starts_with(key) {
                            Some((k.clone(), &item.value))
                        } else {
                            None
                        }
                    }
                    Filter::EndWith(ref key) => {
                        if k.ends_with(key) {
                            Some((k.clone(), &item.value))
                        } else {
                            None
                        }
                    }
                    Filter::StartAndEndWith(ref start_key, ref end_key) => {
                        if k.starts_with(start_key) && k.ends_with(end_key) {
                            Some((k.clone(), &item.value))
                        } else {
                            None
                        }
                    }
                    Filter::None => Some((k.clone(), &item.value)),
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

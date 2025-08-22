//! Event system for cache operation notifications.
//!
//! This module provides an event system that allows you to receive notifications
//! when cache operations occur, such as insertions, removals, or cache clearing.

use crate::cache::Key;
use valu3::value::Value;

/// Represents different types of cache events.
///
/// Events are sent through a channel when cache operations occur, allowing
/// external observers to react to cache changes.
///
/// # Examples
///
/// ```
/// use quickleaf::{Event, EventData};
/// use quickleaf::Cache;
/// use quickleaf::valu3::traits::ToValueBehavior;
/// use std::sync::mpsc::channel;
///
/// let (tx, rx) = channel();
/// let mut cache = Cache::with_sender(5, tx);
///
/// // Insert an item
/// cache.insert("user_123", "session_data");
///
/// // Receive the insert event
/// if let Ok(event) = rx.try_recv() {
///     match event {
///         Event::Insert(data) => {
///             println!("Inserted: {} = {}", data.key, data.value);
///             assert_eq!(data.key, "user_123");
///         },
///         Event::Remove(data) => {
///             println!("Removed: {} = {}", data.key, data.value);
///         },
///         Event::Clear => {
///             println!("Cache cleared");
///         },
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// An item was inserted into the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::{Event, EventData};
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let event = Event::insert("key".to_string(), "value".to_value());
    /// match event {
    ///     Event::Insert(data) => {
    ///         assert_eq!(data.key, "key");
    ///         assert_eq!(data.value, "value".to_value());
    ///     },
    ///     _ => panic!("Expected insert event"),
    /// }
    /// ```
    Insert(EventData),

    /// An item was removed from the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::{Event, EventData};
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let event = Event::remove("key".to_string(), "value".to_value());
    /// match event {
    ///     Event::Remove(data) => {
    ///         assert_eq!(data.key, "key");
    ///         assert_eq!(data.value, "value".to_value());
    ///     },
    ///     _ => panic!("Expected remove event"),
    /// }
    /// ```
    Remove(EventData),

    /// The entire cache was cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Event;
    ///
    /// let event = Event::clear();
    /// match event {
    ///     Event::Clear => println!("Cache was cleared"),
    ///     _ => panic!("Expected clear event"),
    /// }
    /// ```
    Clear,
}

/// Data associated with cache insert and remove events.
///
/// Contains the key and value involved in the operation.
///
/// # Examples
///
/// ```
/// use quickleaf::EventData;
/// use quickleaf::valu3::traits::ToValueBehavior;
///
/// let event_data = EventData {
///     key: "session_id".to_string(),
///     value: "abc123".to_value(),
/// };
///
/// assert_eq!(event_data.key, "session_id");
/// assert_eq!(event_data.value, "abc123".to_value());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct EventData {
    /// The key associated with the event.
    pub key: Key,
    /// The value associated with the event.
    pub value: Value,
}

impl Event {
    /// Creates a new insert event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Event;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let event = Event::insert("user_session".to_string(), "active".to_value());
    ///
    /// match event {
    ///     Event::Insert(data) => {
    ///         assert_eq!(data.key, "user_session");
    ///         assert_eq!(data.value, "active".to_value());
    ///     },
    ///     _ => panic!("Expected insert event"),
    /// }
    /// ```
    pub fn insert(key: Key, value: Value) -> Self {
        Self::Insert(EventData { key, value })
    }

    /// Creates a new remove event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Event;
    /// use quickleaf::valu3::traits::ToValueBehavior;
    ///
    /// let event = Event::remove("expired_key".to_string(), "old_data".to_value());
    ///
    /// match event {
    ///     Event::Remove(data) => {
    ///         assert_eq!(data.key, "expired_key");
    ///         assert_eq!(data.value, "old_data".to_value());
    ///     },
    ///     _ => panic!("Expected remove event"),
    /// }
    /// ```
    pub fn remove(key: Key, value: Value) -> Self {
        Self::Remove(EventData { key, value })
    }

    /// Creates a new clear event.
    ///
    /// # Examples
    ///
    /// ```
    /// use quickleaf::Event;
    ///
    /// let event = Event::clear();
    ///
    /// match event {
    ///     Event::Clear => println!("Cache was cleared"),
    ///     _ => panic!("Expected clear event"),
    /// }
    /// ```
    pub fn clear() -> Self {
        Self::Clear
    }
}

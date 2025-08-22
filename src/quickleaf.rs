//! Main cache type alias for the Quickleaf library.
//!
//! This module provides the main `Quickleaf` type, which is an alias for the `Cache` struct.

use crate::Cache;

/// Main cache type for the Quickleaf library.
///
/// `Quickleaf` is a type alias for `Cache`, providing the same functionality
/// with a more brand-focused name. Use this type when you want to emphasize
/// that you're using the Quickleaf caching library.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// use quickleaf::Quickleaf;
/// use quickleaf::valu3::traits::ToValueBehavior;
///
/// let mut cache = Quickleaf::new(100);
/// cache.insert("user_123", "session_data");
///
/// assert_eq!(cache.get("user_123"), Some(&"session_data".to_value()));
/// assert_eq!(cache.len(), 1);
/// ```
///
/// ## With TTL Support
///
/// ```
/// use quickleaf::Quickleaf;
/// use quickleaf::valu3::traits::ToValueBehavior;
/// use std::time::Duration;
///
/// let mut cache = Quickleaf::with_default_ttl(50, Duration::from_secs(300));
/// cache.insert("session", "active");  
/// cache.insert_with_ttl("temp", "data", Duration::from_secs(60));  
///
/// assert!(cache.contains_key("session"));
/// ```
///
/// ## With Event Notifications
///
/// ```
/// use quickleaf::{Quickleaf, Event};
/// use quickleaf::valu3::traits::ToValueBehavior;
/// use std::sync::mpsc::channel;
///
/// let (tx, rx) = channel();
/// let mut cache = Quickleaf::with_sender(10, tx);
///
/// cache.insert("monitor", "this");
///
/// // Receive the insert event
/// if let Ok(event) = rx.try_recv() {
///     match event {
///         Event::Insert(data) => {
///             assert_eq!(data.key, "monitor");
///             assert_eq!(data.value, "this".to_value());
///         },
///         _ => panic!("Expected insert event"),
///     }
/// }
/// ```
pub type Quickleaf = Cache;

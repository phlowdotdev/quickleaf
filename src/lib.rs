//! # Quickleaf Cache
//!
//! Quickleaf Cache is a Rust library that provides a simple and efficient in-memory cache with support for filtering, ordering, limiting results, TTL (Time To Live), event notifications, and optional persistent storage. It is designed to be lightweight and easy to use.
//!
//! ## Features
//!
//! - Insert and remove key-value pairs
//! - Retrieve values by key
//! - Clear the cache
//! - List cache entries with support for filtering, ordering, and limiting results
//! - **TTL (Time To Live) support** with lazy cleanup
//! - **Persistent storage** using SQLite (optional feature)
//! - Custom error handling
//! - Event notifications for cache operations
//! - Support for generic values using [valu3](https:
//!
//! ## Installation
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! quickleaf = "0.4"
//!
//! # For persistence support (optional)
//! quickleaf = { version = "0.4", features = ["persist"] }
//! ```
//!
//! ## Usage
//!
//! Here's a basic example of how to use Quickleaf Cache:
//!
//! ```rust
//! use quickleaf::{Quickleaf, ListProps, Order, Filter, prelude::*};
//! use quickleaf::valu3::value::Value;
//!
//! fn main() {
//!     let mut cache = Quickleaf::new(2);
//!     cache.insert("key1", 1);
//!     cache.insert("key2", 2);
//!     cache.insert("key3", 3);
//!
//!     assert_eq!(cache.get("key1"), None);
//!     assert_eq!(cache.get("key2"), Some(&2.to_value()));
//!     assert_eq!(cache.get("key3"), Some(&3.to_value()));
//!
//!     let list_props = ListProps::default()
//!         .order(Order::Asc);
//!
//!     let result = cache.list(list_props).unwrap();
//!     for (key, value) in result {
//!         println!("{}: {}", key, value);
//!     }
//! }
//! ```
//!
//! ### Using Filters
//!
//! You can use filters to narrow down the results when listing cache entries. Here are some examples:
//!
//! #### Filter by Start With
//!
//! ```rust
//! use quickleaf::{Quickleaf, ListProps, Order, Filter};
//!
//!
//! fn main() {
//!     let mut cache = Quickleaf::new(10);
//!     cache.insert("apple", 1);
//!     cache.insert("banana", 2);
//!     cache.insert("apricot", 3);
//!
//!     let list_props = ListProps::default()
//!         .order(Order::Asc)
//!         .filter(Filter::StartWith("ap".to_string()));
//!
//!     let result = cache.list(list_props).unwrap();
//!     for (key, value) in result {
//!         println!("{}: {}", key, value);
//!     }
//! }
//! ```
//!
//! #### Filter by End With
//!
//! ```rust
//! use quickleaf::{Quickleaf, ListProps, Order, Filter};
//!
//! fn main() {
//!     let mut cache = Quickleaf::new(10);
//!     cache.insert("apple", 1);
//!     cache.insert("banana", 2);
//!     cache.insert("pineapple", 3);
//!
//!     let list_props = ListProps::default()
//!         .order(Order::Asc)
//!         .filter(Filter::EndWith("apple".to_string()));
//!
//!     let result = cache.list(list_props).unwrap();
//!     for (key, value) in result {
//!         println!("{}: {}", key, value);
//!     }
//! }
//! ```
//!
//! #### Filter by Start And End With
//!
//! ```rust
//! use quickleaf::{Quickleaf, ListProps, Order, Filter};
//!
//! fn main() {
//!     let mut cache = Quickleaf::new(10);
//!     cache.insert("applemorepie", 1);
//!     cache.insert("banana", 2);
//!     cache.insert("pineapplepie", 3);
//!
//!     let list_props = ListProps::default()
//!         .order(Order::Asc)
//!         .filter(Filter::StartAndEndWith("apple".to_string(), "pie".to_string()));
//!
//!     let result = cache.list(list_props).unwrap();
//!     for (key, value) in result {
//!         println!("{}: {}", key, value);
//!     }
//! }
//! ```
//!
//! ### Using TTL (Time To Live)
//!
//! You can set TTL for cache entries to automatically expire them after a certain duration:
//!
//! ```rust
//! use quickleaf::{Quickleaf, Duration};
//!
//! fn main() {
//!     let mut cache = Quickleaf::new(10);
//!     
//!     
//!     cache.insert_with_ttl("session", "user_data", Duration::from_secs(5));
//!     
//!     
//!     let mut cache_with_default = Quickleaf::with_default_ttl(10, Duration::from_secs(60));
//!     cache_with_default.insert("key", "value");
//!     
//!     
//!     let removed_count = cache.cleanup_expired();
//!     println!("Removed {} expired items", removed_count);
//! }
//! ```
//!
//! ### Using Events
//!
//! You can use events to get notified when cache entries are inserted, removed, or cleared. Here is an example:
//!
//! ```rust
//! use quickleaf::{Quickleaf, Event, prelude::*};
//! use std::sync::mpsc::channel;
//! use quickleaf::valu3::value::Value;
//!
//! fn main() {
//!     let (tx, rx) = channel();
//!     let mut cache = Quickleaf::with_sender(10, tx);
//!
//!     cache.insert("key1", 1);
//!     cache.insert("key2", 2);
//!     cache.insert("key3", 3);
//!
//!     let mut items = Vec::new();
//!
//!     for data in rx {
//!         items.push(data);
//!
//!         if items.len() == 3 {
//!             break;
//!         }
//!     }
//!
//!     assert_eq!(items.len(), 3);
//!     assert_eq!(
//!         items[0],
//!         Event::insert("key1".to_string(), 1.to_value())
//!     );
//!     assert_eq!(
//!         items[1],
//!         Event::insert("key2".to_string(), 2.to_value())
//!     );
//!     assert_eq!(
//!         items[2],
//!         Event::insert("key3".to_string(), 3.to_value())
//!     );
//! }
//! ```
//!
//! ### Event Types
//!
//! There are three types of events:
//!
//! 1. `Insert`: Triggered when a new entry is inserted into the cache.
//! 2. `Remove`: Triggered when an entry is removed from the cache.
//! 3. `Clear`: Triggered when the cache is cleared.
//!
//! ## Persistent Storage (Optional)
//!
//! Quickleaf supports optional persistent storage using SQLite as a backing store. This feature
//! provides durability across application restarts while maintaining high-performance in-memory operations.
//!
//! ### Enabling Persistence
//!
//! Add the `persist` feature to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! quickleaf = { version = "0.4", features = ["persist"] }
//! ```
//!
//! ### Basic Persistent Cache
//!
//! ```rust,no_run
//! # #[cfg(feature = "persist")]
//! # {
//! use quickleaf::Cache;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut cache = Cache::with_persist("cache.db", 1000)?;
//!     
//!     cache.insert("user:123", "Alice");
//!     cache.insert("user:456", "Bob");
//!     
//!     drop(cache);
//!     
//!     let mut cache = Cache::with_persist("cache.db", 1000)?;
//!     
//!     println!("{:?}", cache.get("user:123"));
//!     
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ### Persistent Cache with TTL
//!
//! ```rust,no_run
//! # #[cfg(feature = "persist")]
//! # {
//! use quickleaf::{Cache, Duration};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut cache = Cache::with_persist("cache.db", 1000)?;
//!     
//!     
//!     cache.insert_with_ttl(
//!         "session:abc",
//!         "temp_data",
//!         Duration::from_secs(3600)
//!     );
//!     
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ### Persistence with Events
//!
//! ```rust,no_run
//! # #[cfg(feature = "persist")]
//! # {
//! use quickleaf::Cache;
//! use std::sync::mpsc::channel;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (tx, rx) = channel();
//!     
//!     let mut cache = Cache::with_persist_and_sender("cache.db", 1000, tx)?;
//!     
//!     cache.insert("key1", "value1");
//!     
//!     for event in rx.try_iter() {
//!         println!("Event: {:?}", event);
//!     }
//!     
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ### Complete Persistence Stack (SQLite + Events + TTL)
//!
//! ```rust,no_run
//! # #[cfg(feature = "persist")]
//! # {
//! use quickleaf::Cache;
//! use std::sync::mpsc::channel;
//! use std::time::Duration;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (tx, rx) = channel();
//!     
//!     let mut cache = Cache::with_persist_and_sender_and_ttl(
//!         "full_featured_cache.db",
//!         1000,
//!         tx,
//!         Duration::from_secs(3600)  
//!     )?;
//!     
//!     cache.insert("session", "user_data");
//!     
//!     cache.insert_with_ttl("temp", "data", Duration::from_secs(60));
//!     
//!     for event in rx.try_iter() {
//!         println!("Event: {:?}", event);
//!     }
//!     
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ### Persistence Features
//!
//! - **Automatic Persistence**: All cache operations are automatically persisted to SQLite
//! - **Background Writer**: Non-blocking write operations using a background thread
//! - **Crash Recovery**: Automatic recovery from unexpected shutdowns
//! - **TTL Preservation**: TTL values are preserved across restarts
//! - **Efficient Storage**: Uses SQLite with optimized indexes for performance
//! - **Seamless Integration**: Works with all existing Quickleaf features

mod cache;
mod error;
mod event;
mod filter;
pub mod filters;
mod list_props;
#[cfg(test)]
#[cfg(feature = "persist")]
mod persist_tests;
mod prefetch;
pub mod prelude;
mod quickleaf;
#[cfg(feature = "persist")]
mod sqlite_store;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod ttl_tests;

pub use cache::{Cache, CacheItem};
pub use error::Error;
pub use event::{Event, EventData};
pub use filter::Filter;
pub use list_props::{ListProps, Order, StartAfter};
pub use quickleaf::Quickleaf;
pub use std::time::Duration;
pub use valu3;
pub use valu3::value::Value;

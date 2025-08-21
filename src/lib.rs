//! # Quickleaf Cache
//!
//! Quickleaf Cache is a Rust library that provides a simple and efficient in-memory cache with support for filtering, ordering, limiting results, TTL (Time To Live), and event notifications. It is designed to be lightweight and easy to use.
//!
//! ## Features
//!
//! - Insert and remove key-value pairs
//! - Retrieve values by key
//! - Clear the cache
//! - List cache entries with support for filtering, ordering, and limiting results
//! - **TTL (Time To Live) support** with lazy cleanup
//! - Custom error handling
//! - Event notifications for cache operations
//! - Support for generic values using [valu3](https://github.com/lowcarboncode/valu3)
//!
//! ## Installation
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! quickleaf = "0.2
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
//!     // Insert with specific TTL (5 seconds)
//!     cache.insert_with_ttl("session", "user_data", Duration::from_secs(5));
//!     
//!     // Insert with default TTL
//!     let mut cache_with_default = Quickleaf::with_default_ttl(10, Duration::from_secs(60));
//!     cache_with_default.insert("key", "value"); // Will expire in 60 seconds
//!     
//!     // Manual cleanup of expired items
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

mod cache;
mod error;
mod event;
mod filter;
mod list_props;
#[cfg(feature = "persist")]
mod sqlite_store;
pub mod prelude;
mod quickleaf;
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

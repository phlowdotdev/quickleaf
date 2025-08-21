# 🍃 Quickleaf Cache

[![Crates.io](https://img.shields.io/crates/v/quickleaf.svg)](https://crates.io/crates/quickleaf)
[![License](https://img.shields.io/crates/l/quickleaf.svg)](https://github.com/lowcarboncode/quickleaf/blob/main/LICENSE)
[![Documentation](https://docs.rs/quickleaf/badge.svg)](https://docs.rs/quickleaf)

Quickleaf Cache is a **fast**, **lightweight**, and **feature-rich** in-memory cache library for Rust. It combines the simplicity of a HashMap with advanced caching features like **TTL (Time To Live)**, **filtering**, **ordering**, and **event notifications**.

## ✨ Features

- 🚀 **High Performance**: O(1) access with ordered key iteration
- ⏰ **TTL Support**: Automatic expiration with lazy cleanup
- 🔍 **Advanced Filtering**: StartWith, EndWith, and complex pattern matching
- 📋 **Flexible Ordering**: Ascending/descending with pagination support
- 🔔 **Event Notifications**: Real-time cache operation events
- 🎯 **LRU Eviction**: Automatic removal of least recently used items
- 💾 **Persistent Storage**: Optional SQLite-backed persistence for durability
- 🛡️ **Type Safety**: Full Rust type safety with generic value support
- 📦 **Lightweight**: Minimal external dependencies

## 📦 Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
quickleaf = "0.4"

# For persistence support (optional)
quickleaf = { version = "0.4", features = ["persist"] }
```

## 🚀 Quick Start

```rust
use quickleaf::{Quickleaf, Duration};

fn main() {
    // Create a cache with capacity of 1000 items
    let mut cache = Quickleaf::new(1000);
    
    // Insert some data
    cache.insert("user:123", "Alice");
    cache.insert("user:456", "Bob");
    
    // Retrieve data
    println!("{:?}", cache.get("user:123")); // Some("Alice")
    
    // Insert with TTL (expires in 60 seconds)
    cache.insert_with_ttl("session:abc", "temp_data", Duration::from_secs(60));
}
```

## 📖 Usage Examples

### Basic Operations

```rust
use quickleaf::Quickleaf;

fn main() {
    let mut cache = Quickleaf::new(5);
    
    // Insert data
    cache.insert("apple", 100);
    cache.insert("banana", 200);
    cache.insert("cherry", 300);
    
    // Get data
    println!("{:?}", cache.get("apple")); // Some(100)
    
    // Check if key exists
    assert!(cache.contains_key("banana"));
    
    // Remove data
    cache.remove("cherry").unwrap();
    
    // Cache info
    println!("Cache size: {}", cache.len());
    println!("Is empty: {}", cache.is_empty());
}
```

### 🕒 TTL (Time To Live) Features

#### Default TTL for All Items

```rust
use quickleaf::{Quickleaf, Duration};

fn main() {
    // Create cache where all items expire after 5 minutes by default
    let mut cache = Quickleaf::with_default_ttl(100, Duration::from_secs(300));
    
    // This item will use the default TTL (5 minutes)
    cache.insert("default_ttl", "expires in 5 min");
    
    // This item has custom TTL (30 seconds)
    cache.insert_with_ttl("custom_ttl", "expires in 30 sec", Duration::from_secs(30));
    
    // Items expire automatically when accessed
    // After 30+ seconds, custom_ttl will return None
    println!("{:?}", cache.get("custom_ttl"));
}
```

#### Manual Cleanup

```rust
use quickleaf::{Quickleaf, Duration};
use std::thread;

fn main() {
    let mut cache = Quickleaf::new(10);
    
    // Add items with short TTL for demo
    cache.insert_with_ttl("temp1", "data1", Duration::from_millis(100));
    cache.insert_with_ttl("temp2", "data2", Duration::from_millis(100));
    cache.insert("permanent", "data3"); // No TTL
    
    println!("Initial size: {}", cache.len()); // 3
    
    // Wait for items to expire
    thread::sleep(Duration::from_millis(150));
    
    // Manual cleanup of expired items
    let removed_count = cache.cleanup_expired();
    println!("Removed {} expired items", removed_count); // 2
    println!("Final size: {}", cache.len()); // 1
}
```

### 🔍 Advanced Filtering

#### Filter by Prefix

```rust
use quickleaf::{Quickleaf, ListProps, Order, Filter};

fn main() {
    let mut cache = Quickleaf::new(10);
    cache.insert("user:123", "Alice");
    cache.insert("user:456", "Bob");
    cache.insert("product:789", "Widget");
    cache.insert("user:999", "Charlie");

    // Get all users (keys starting with "user:")
    let users = cache.list(
        ListProps::default()
            .filter(Filter::StartWith("user:".to_string()))
            .order(Order::Asc)
    ).unwrap();

    for (key, value) in users {
        println!("{}: {}", key, value);
    }
}
```

#### Filter by Suffix

```rust
use quickleaf::{Quickleaf, ListProps, Filter};

fn main() {
    let mut cache = Quickleaf::new(10);
    cache.insert("config.json", "{}");
    cache.insert("data.json", "[]");
    cache.insert("readme.txt", "docs");
    cache.insert("settings.json", "{}");

    // Get all JSON files
    let json_files = cache.list(
        ListProps::default()
            .filter(Filter::EndWith(".json".to_string()))
    ).unwrap();

    println!("JSON files found: {}", json_files.len()); // 3
}
```

#### Complex Pattern Filtering

```rust
use quickleaf::{Quickleaf, ListProps, Filter, Order};

fn main() {
    let mut cache = Quickleaf::new(10);
    cache.insert("cache_user_data", "user1");
    cache.insert("cache_product_info", "product1");
    cache.insert("temp_user_session", "session1");
    cache.insert("cache_user_preferences", "prefs1");

    // Get cached user data (starts with "cache_" and ends with "_data")
    let cached_user_data = cache.list(
        ListProps::default()
            .filter(Filter::StartAndEndWith("cache_".to_string(), "_data".to_string()))
            .order(Order::Desc)
    ).unwrap();

    for (key, value) in cached_user_data {
        println!("{}: {}", key, value);
    }
}
```

### 📋 Pagination and Ordering

```rust
use quickleaf::{Quickleaf, ListProps, Order};

fn main() {
    let mut cache = Quickleaf::new(100);
    
    // Add some test data
    for i in 1..=20 {
        cache.insert(format!("item_{:02}", i), i);
    }
    
    // Get first 5 items in ascending order
    let page1 = cache.list(
        ListProps::default()
            .order(Order::Asc)
    ).unwrap();
    
    println!("First 5 items:");
    for (i, (key, value)) in page1.iter().take(5).enumerate() {
        println!("  {}: {} = {}", i+1, key, value);
    }
    
    // Get top 3 items in descending order
    let desc_items = cache.list(
        ListProps::default()
            .order(Order::Desc)
    ).unwrap();
    
    println!("Top 3 items (desc):");
    for (key, value) in desc_items.iter().take(3) {
        println!("  {}: {}", key, value);
    }
}
```

### 💾 Persistent Cache (SQLite Backend)

Quickleaf supports optional persistence using SQLite as a backing store. This provides durability across application restarts while maintaining the same high-performance in-memory operations.

#### Basic Persistent Cache

```rust
use quickleaf::Cache;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a persistent cache backed by SQLite
    let mut cache = Cache::with_persist("cache.db", 1000)?;
    
    // Insert data - automatically persisted
    cache.insert("user:123", "Alice");
    cache.insert("user:456", "Bob");
    
    // Data survives application restart
    drop(cache);
    
    // Later or after restart...
    let mut cache = Cache::with_persist("cache.db", 1000)?;
    
    // Data is still available
    println!("{:?}", cache.get("user:123")); // Some("Alice")
    
    Ok(())
}
```

#### Persistent Cache with TTL

```rust
use quickleaf::{Cache, Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Option 1: Use with_persist and insert items with individual TTL
    let mut cache = Cache::with_persist("cache.db", 1000)?;
    cache.insert_with_ttl(
        "session:abc", 
        "temp_data", 
        Duration::from_secs(3600)
    );
    
    // Option 2: Use with_persist_and_ttl for default TTL on all items
    let mut cache_with_default = Cache::with_persist_and_ttl(
        "cache_with_ttl.db",
        1000,
        Duration::from_secs(300)  // 5 minutes default TTL
    )?;
    
    // This item will use the default TTL (5 minutes)
    cache_with_default.insert("auto_expire", "data");
    
    // You can still override with custom TTL
    cache_with_default.insert_with_ttl(
        "custom_expire",
        "data",
        Duration::from_secs(60)  // 1 minute instead of default 5
    );
    
    // TTL is preserved across restarts
    // Expired items are automatically cleaned up on load
    
    Ok(())
}
```

#### Persistence Features

- **Automatic Persistence**: All cache operations are automatically persisted
- **Background Writer**: Non-blocking write operations using a background thread
- **Crash Recovery**: Automatic recovery from unexpected shutdowns
- **TTL Preservation**: TTL values are preserved across restarts
- **Efficient Storage**: Uses SQLite with optimized indexes for performance
- **Compatibility**: Works seamlessly with all existing Quickleaf features

#### Available Persistence Constructors

| Constructor | Description | Use Case |
|------------|-------------|----------|
| `with_persist(path, capacity)` | Basic persistent cache | Simple persistence without events |
| `with_persist_and_ttl(path, capacity, ttl)` | Persistent cache with default TTL | Session stores, temporary data with persistence |
| `with_persist_and_sender(path, capacity, sender)` | Persistent cache with events | Monitoring, logging, real-time updates |

### 🔔 Event Notifications

```rust
use quickleaf::{Quickleaf, Event};
use std::sync::mpsc::channel;
use std::thread;

fn main() {
    let (tx, rx) = channel();
    let mut cache = Quickleaf::with_sender(10, tx);
    
    // Spawn a thread to handle events
    let event_handler = thread::spawn(move || {
        for event in rx {
            match event {
                Event::Insert(data) => {
                    println!("➕ Inserted: {} = {}", data.key, data.value);
                }
                Event::Remove(data) => {
                    println!("➖ Removed: {} = {}", data.key, data.value);
                }
                Event::Clear => {
                    println!("🗑️ Cache cleared");
                }
            }
        }
    });
    
    // Perform cache operations (will trigger events)
    cache.insert("user:1", "Alice");
    cache.insert("user:2", "Bob");
    cache.remove("user:1").unwrap();
    cache.clear();
    
    // Close the sender to stop the event handler
    drop(cache);
    event_handler.join().unwrap();
}
```

### 🔄 Combined Features Example

```rust
use quickleaf::{Quickleaf, Duration, ListProps, Filter, Order};
use std::thread;

fn main() {
    // Create cache with default TTL and event notifications
    let (tx, _rx) = std::sync::mpsc::channel();
    let mut cache = Quickleaf::with_sender_and_ttl(50, tx, Duration::from_secs(300));
    
    // Insert user sessions with custom TTLs
    cache.insert_with_ttl("session:guest", "temporary", Duration::from_secs(30));
    cache.insert_with_ttl("session:user123", "authenticated", Duration::from_secs(3600));
    cache.insert("config:theme", "dark"); // Uses default TTL
    cache.insert("config:lang", "en");    // Uses default TTL
    
    // Get all active sessions
    let sessions = cache.list(
        ListProps::default()
            .filter(Filter::StartWith("session:".to_string()))
            .order(Order::Asc)
    ).unwrap();
    
    println!("Active sessions: {}", sessions.len());
    
    // Simulate time passing
    thread::sleep(Duration::from_secs(35));
    
    // Guest session should be expired now
    println!("Guest session: {:?}", cache.get("session:guest")); // None
    println!("User session: {:?}", cache.get("session:user123")); // Some(...)
    
    // Manual cleanup
    let expired_count = cache.cleanup_expired();
    println!("Cleaned up {} expired items", expired_count);
}
```

## 🏗️ Architecture

### Cache Structure

Quickleaf uses a dual-structure approach for optimal performance:

- **HashMap**: O(1) key-value access
- **Vec**: Maintains sorted key order for efficient iteration
- **Lazy Cleanup**: TTL items are removed when accessed, not proactively
- **SQLite Backend** (optional): Provides durable storage with background persistence

### TTL Strategy

- **Lazy Cleanup**: Expired items are removed during access operations (`get`, `contains_key`, `list`)
- **Manual Cleanup**: Use `cleanup_expired()` for proactive cleaning
- **No Background Threads**: Zero overhead until items are accessed (except for optional persistence)

### Persistence Architecture (Optional)

When persistence is enabled:

- **In-Memory First**: All operations work on the in-memory cache for speed
- **Background Writer**: A separate thread handles SQLite writes asynchronously
- **Event-Driven**: Cache operations trigger persistence events
- **Auto-Recovery**: On startup, cache is automatically restored from SQLite
- **Expired Cleanup**: Expired items are filtered out during load

## 🔧 API Reference

### Cache Creation

```rust
// Basic cache
let cache = Quickleaf::new(capacity);

// With default TTL
let cache = Quickleaf::with_default_ttl(capacity, ttl);

// With event notifications
let cache = Quickleaf::with_sender(capacity, sender);

// With both TTL and events
let cache = Quickleaf::with_sender_and_ttl(capacity, sender, ttl);

// With persistence (requires "persist" feature)
let cache = Cache::with_persist("cache.db", capacity)?;

// With persistence and default TTL
let cache = Cache::with_persist_and_ttl("cache.db", capacity, ttl)?;

// With persistence and events
let cache = Cache::with_persist_and_sender("cache.db", capacity, sender)?;
```

### Core Operations

```rust
// Insert operations
cache.insert(key, value);
cache.insert_with_ttl(key, value, ttl);

// Access operations
cache.get(key);           // Returns Option<&Value>
cache.get_mut(key);       // Returns Option<&mut Value>
cache.contains_key(key);  // Returns bool

// Removal operations
cache.remove(key);        // Returns Result<(), Error>
cache.clear();            // Removes all items

// TTL operations
cache.cleanup_expired();  // Returns count of removed items
cache.set_default_ttl(ttl);
cache.get_default_ttl();
```

### Filtering and Listing

```rust
// List operations
cache.list(props);        // Returns Result<Vec<(Key, &Value)>, Error>

// Filter types
Filter::None
Filter::StartWith(prefix)
Filter::EndWith(suffix)
Filter::StartAndEndWith(prefix, suffix)

// Ordering
Order::Asc    // Ascending
Order::Desc   // Descending
```

## 🧪 Testing

Run the test suite:

```bash
# All tests
cargo test

# TTL-specific tests
cargo test ttl

# With output
cargo test -- --nocapture
```

## 📊 Performance

### Benchmarks

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Insert | O(log n) | Due to ordered insertion |
| Get | O(1) | HashMap lookup |
| Remove | O(n) | Vec removal |
| List | O(n) | Iteration with filtering |
| TTL Check | O(1) | Simple time comparison |

### Memory Usage

- **Base overhead**: ~48 bytes per cache instance
- **Per item**: ~(key_size + value_size + 56) bytes
- **TTL overhead**: +24 bytes per item with TTL

## 📚 Examples

Check out the `examples/` directory for more comprehensive examples:

```bash
# Run the TTL example
cargo run --example ttl_example

# Run the persistence example
cargo run --example test_persist --features persist

# Run the interactive TUI with persistence
cargo run --example tui_interactive --features tui-example
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development

```bash
# Clone the repository
git clone https://github.com/lowcarboncode/quickleaf.git
cd quickleaf

# Run tests
cargo test

# Run examples
cargo run --example ttl_example

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

## 📄 License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- [Documentation](https://docs.rs/quickleaf)
- [Crates.io](https://crates.io/crates/quickleaf)
- [Repository](https://github.com/lowcarboncode/quickleaf)
- [Issues](https://github.com/lowcarboncode/quickleaf/issues)

---

**Made with ❤️ by the LowCarbonCode team**

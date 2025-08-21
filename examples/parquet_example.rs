//! Example demonstrating Parquet persistence feature
//! 
//! This example requires the "parquet" feature to be enabled:
//! cargo run --example parquet_example --features parquet

#[cfg(feature = "parquet")]
use quickleaf::{Quickleaf, ListProps, Order, Filter, Duration};
#[cfg(feature = "parquet")]
use std::thread;
#[cfg(feature = "parquet")]
use std::path::Path;

#[cfg(feature = "parquet")]
fn main() {
    println!("🍃 Quickleaf Parquet Persistence Example");
    println!("=========================================\n");
    
    let parquet_path = "cache_data.parquet";
    
    // Check if the file already exists from a previous run
    if Path::new(parquet_path).exists() {
        println!("📂 Found existing Parquet file, loading previous data...\n");
    } else {
        println!("📝 Creating new Parquet file for persistence...\n");
    }
    
    // Create cache with Parquet persistence
    let mut cache = Quickleaf::with_parquet(parquet_path, 100)
        .expect("Failed to create cache with Parquet");
    
    // Check if we have any existing data
    let existing_items = cache.list(ListProps::default()).unwrap();
    if !existing_items.is_empty() {
        println!("📋 Loaded {} items from Parquet:", existing_items.len());
        for (key, value) in existing_items.iter().take(5) {
            println!("   - {}: {}", key, value);
        }
        if existing_items.len() > 5 {
            println!("   ... and {} more items", existing_items.len() - 5);
        }
        println!();
    }
    
    // Add some new data
    println!("➕ Adding new data to cache...");
    cache.insert("user:alice", "Alice Johnson");
    cache.insert("user:bob", "Bob Smith");
    cache.insert("session:abc123", "active_session");
    cache.insert("config:theme", "dark");
    cache.insert("config:language", "en-US");
    
    println!("   Added 5 new items");
    println!("   Total cache size: {}\n", cache.len());
    
    // Demonstrate filtering
    println!("🔍 Filtering users:");
    let users = cache.list(
        ListProps::default()
            .filter(Filter::StartWith("user:".to_string()))
            .order(Order::Asc)
    ).unwrap();
    
    for (key, value) in users {
        println!("   - {}: {}", key, value);
    }
    
    // Remove an item
    println!("\n➖ Removing 'session:abc123'...");
    cache.remove("session:abc123").unwrap();
    
    // Clear message for persistence
    println!("\n💾 All operations are automatically persisted to: {}", parquet_path);
    println!("   Try running this example again to see data persistence!");
    
    // Give the background writer a moment to flush
    thread::sleep(Duration::from_millis(100));
    
    println!("\n✅ Example completed!");
}

#[cfg(not(feature = "parquet"))]
fn main() {
    println!("❌ This example requires the 'parquet' feature to be enabled.");
    println!("   Run with: cargo run --example parquet_example --features parquet");
}

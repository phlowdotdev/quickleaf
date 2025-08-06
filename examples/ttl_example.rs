use quickleaf::{Quickleaf, ListProps, Order, Filter, Duration};
use std::thread;

fn main() {
    println!("🍃 Quickleaf TTL Cache Example");
    println!("================================\n");
    
    // Create cache with default TTL
    let mut cache = Quickleaf::with_default_ttl(5, Duration::from_secs(2));
    
    // Insert some data with different TTL strategies
    println!("📝 Inserting data...");
    cache.insert("persistent", "This won't expire"); // Uses default TTL (2 seconds)
    cache.insert_with_ttl("short_lived", "This expires in 1 second", Duration::from_secs(1));
    cache.insert_with_ttl("medium_lived", "This expires in 3 seconds", Duration::from_secs(3));
    
    println!("   - persistent: {}", cache.get("persistent").unwrap());
    println!("   - short_lived: {}", cache.get("short_lived").unwrap());
    println!("   - medium_lived: {}", cache.get("medium_lived").unwrap());
    println!("   Cache size: {}\n", cache.len());
    
    // Wait 1.5 seconds
    println!("⏱️  Waiting 1.5 seconds...");
    thread::sleep(Duration::from_millis(1500));
    
    // Check what's still available
    println!("🔍 Checking cache after 1.5 seconds:");
    println!("   - persistent: {:?}", cache.get("persistent"));
    println!("   - short_lived: {:?}", cache.get("short_lived")); // Should be None (expired)
    println!("   - medium_lived: {:?}", cache.get("medium_lived"));
    println!("   Cache size: {}\n", cache.len());
    
    // Wait another 2 seconds
    println!("⏱️  Waiting another 2 seconds...");
    thread::sleep(Duration::from_secs(2));
    
    // Manual cleanup
    println!("🧹 Manual cleanup:");
    let removed_count = cache.cleanup_expired();
    println!("   Removed {} expired items", removed_count);
    println!("   Cache size: {}\n", cache.len());
    
    // List remaining items
    println!("📋 Remaining items:");
    let result = cache.list(ListProps::default()).unwrap();
    for (key, value) in result {
        println!("   - {}: {}", key, value);
    }
    
    // Demonstrate filtering with TTL
    println!("\n🔧 Adding more test data...");
    cache.insert_with_ttl("apple_pie", "Delicious!", Duration::from_secs(10));
    cache.insert_with_ttl("apple_juice", "Refreshing!", Duration::from_secs(10));
    cache.insert_with_ttl("banana_split", "Sweet!", Duration::from_secs(10));
    
    println!("🔍 Filtering items starting with 'apple':");
    let filtered = cache.list(
        ListProps::default()
            .filter(Filter::StartWith("apple".to_string()))
            .order(Order::Asc)
    ).unwrap();
    
    for (key, value) in filtered {
        println!("   - {}: {}", key, value);
    }
    
    println!("\n✅ Example completed!");
}

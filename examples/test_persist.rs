//! Test SQLite persistence

#[cfg(feature = "persist")]
use quickleaf::{Cache, ListProps};

#[cfg(feature = "persist")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = "test_cache.db";

    // Remove old file if exists
    let _ = std::fs::remove_file(test_file);

    // Test 1: Create cache and insert data
    println!("Test 1: Creating cache and inserting data...");
    {
        let mut cache = Cache::with_persist(test_file, 100)?;
        cache.insert("key1", "value1");
        cache.insert("key2", "value2");
        cache.insert("key3", "value3");
        println!("Inserted 3 items");

        // Give time for background writer to persist
        thread::sleep(Duration::from_secs(2));
    }

    // Test 2: Load cache from file
    println!("\nTest 2: Loading cache from file...");
    {
        let mut cache = Cache::with_persist(test_file, 100)?;

        // Check if data was persisted
        if let Some(val) = cache.get("key1") {
            println!("✓ Found key1: {:?}", val);
        } else {
            println!("✗ key1 not found!");
        }

        if let Some(val) = cache.get("key2") {
            println!("✓ Found key2: {:?}", val);
        } else {
            println!("✗ key2 not found!");
        }

        if let Some(val) = cache.get("key3") {
            println!("✓ Found key3: {:?}", val);
        } else {
            println!("✗ key3 not found!");
        }

        // Add more data
        cache.insert("key4", "value4");
        println!("Added key4");

        thread::sleep(Duration::from_secs(2));
    }

    // Test 3: Verify all data
    println!("\nTest 3: Final verification...");
    {
        let mut cache = Cache::with_persist(test_file, 100)?;

        let items = cache.list(ListProps::default())?;
        println!("Total items in cache: {}", items.len());

        for (key, value) in items {
            println!("  {} = {:?}", key, value);
        }
    }

    // Clean up
    // let _ = std::fs::remove_file(test_file);

    println!("\n✅ Persistence test completed!");
    Ok(())
}

#[cfg(not(feature = "persist"))]
fn main() {
    println!("This example requires the 'persist' feature");
}

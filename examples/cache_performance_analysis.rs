use std::time::Instant;
use quickleaf::Cache;

fn main() {
    println!("Testing real-world cache performance scenarios...");
    
    // Test 1: Random access patterns (where prefetch should hurt)
    test_random_access();
    
    // Test 2: Sequential access patterns (where prefetch should help)
    test_sequential_access();
    
    // Test 3: Large cache list operations
    test_large_list_operations();
    
    // Test 4: Cleanup operations with many expired items
    test_cleanup_operations();
}

fn test_random_access() {
    println!("\n--- Random Access Test ---");
    let mut cache = Cache::new(10000);
    
    // Pre-populate with 10k items
    for i in 0..10000 {
        cache.insert(format!("item{:05}", i), format!("value{}", i));
    }
    
    // Random access pattern
    let mut rng_seed = 42u64;
    let iterations = 100000;
    
    let start = Instant::now();
    for _ in 0..iterations {
        // Simple LCG for reproducible "random" numbers
        rng_seed = rng_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let index = (rng_seed % 10000) as usize;
        let key = format!("item{:05}", index);
        std::hint::black_box(cache.get(&key));
    }
    let duration = start.elapsed();
    
    println!("Random access ({} ops): {:?}", iterations, duration);
    println!("Average per access: {:?}", duration / iterations);
}

fn test_sequential_access() {
    println!("\n--- Sequential Access Test ---");
    let mut cache = Cache::new(10000);
    
    // Pre-populate
    for i in 0..10000 {
        cache.insert(format!("seq{:05}", i), format!("value{}", i));
    }
    
    let iterations = 10;
    
    let start = Instant::now();
    for _ in 0..iterations {
        // Sequential access through the entire cache
        for i in 0..10000 {
            let key = format!("seq{:05}", i);
            std::hint::black_box(cache.get(&key));
        }
    }
    let duration = start.elapsed();
    
    println!("Sequential access ({} full sweeps): {:?}", iterations, duration);
    println!("Average per access: {:?}", duration / (iterations * 10000));
}

fn test_large_list_operations() {
    println!("\n--- Large List Operations Test ---");
    let mut cache = Cache::new(50000);
    
    // Pre-populate with 50k items
    for i in 0..50000 {
        cache.insert(format!("list{:06}", i), i);
    }
    
    let iterations = 100;
    
    let start = Instant::now();
    for _ in 0..iterations {
        let mut props = quickleaf::ListProps::default();
        props.limit = 1000; // Get 1000 items each time
        std::hint::black_box(cache.list(props).unwrap());
    }
    let duration = start.elapsed();
    
    println!("List operations ({} iterations, 1000 items each): {:?}", iterations, duration);
    println!("Average per list operation: {:?}", duration / iterations);
}

fn test_cleanup_operations() {
    println!("\n--- Cleanup Operations Test ---");
    let mut cache = Cache::new(20000);
    
    // Pre-populate with mix of expired and valid items
    for i in 0..10000 {
        // Add expired items (very short TTL)
        cache.insert_with_ttl(
            format!("expired{:05}", i), 
            format!("value{}", i),
            std::time::Duration::from_nanos(1)
        );
    }
    
    // Add some valid items
    for i in 10000..20000 {
        cache.insert(format!("valid{:05}", i), format!("value{}", i));
    }
    
    // Wait a bit to ensure expiration
    std::thread::sleep(std::time::Duration::from_millis(1));
    
    let iterations = 1000;
    
    let start = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(cache.cleanup_expired());
    }
    let duration = start.elapsed();
    
    println!("Cleanup operations ({} iterations): {:?}", iterations, duration);
    println!("Average per cleanup: {:?}", duration / iterations);
}

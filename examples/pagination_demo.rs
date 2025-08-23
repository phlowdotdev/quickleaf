//! Example demonstrating pagination with limit and start_after_key
//!
//! Run with: cargo run --example pagination_demo

use quickleaf::{Cache, ListProps, Order};

fn main() {
    // Create a cache with capacity for 100 items
    let mut cache = Cache::new(100);

    // Insert 30 items for demonstration
    println!("📝 Inserting 30 items into cache...");
    for i in 0..30 {
        cache.insert(format!("item_{:03}", i), format!("value_{}", i));
    }

    println!("\n✅ Cache now contains {} items\n", cache.len());

    // Example 1: Basic pagination with limit
    println!("📄 Example 1: Basic pagination with limit=5");
    println!("----------------------------------------");

    let props = ListProps::default().order(Order::Asc).limit(5);

    let page1 = cache.list(props).unwrap();
    println!("First page ({} items):", page1.len());
    for (key, value) in &page1 {
        println!("  • {} = {:?}", key, value);
    }

    // Example 2: Getting the next page using start_after_key
    println!("\n📄 Example 2: Next page using start_after_key");
    println!("--------------------------------------------");

    let last_key_page1 = &page1.last().unwrap().0;
    println!("Continuing after key: '{}'", last_key_page1);

    let props = ListProps::default()
        .order(Order::Asc)
        .start_after_key(last_key_page1)
        .limit(5);

    let page2 = cache.list(props).unwrap();
    println!("Second page ({} items):", page2.len());
    for (key, value) in &page2 {
        println!("  • {} = {:?}", key, value);
    }

    // Example 3: Pagination with different limit
    println!("\n📄 Example 3: Larger pages with limit=10");
    println!("----------------------------------------");

    let props = ListProps::default().order(Order::Asc).limit(10);

    let large_page = cache.list(props).unwrap();
    println!("Large page ({} items):", large_page.len());
    println!("First item: {} = {:?}", large_page[0].0, large_page[0].1);
    println!(
        "Last item: {} = {:?}",
        large_page.last().unwrap().0,
        large_page.last().unwrap().1
    );

    // Example 4: Descending order pagination
    println!("\n📄 Example 4: Descending order pagination");
    println!("----------------------------------------");

    let props = ListProps::default().order(Order::Desc).limit(5);

    let desc_page = cache.list(props).unwrap();
    println!("Descending page ({} items):", desc_page.len());
    for (key, value) in &desc_page {
        println!("  • {} = {:?}", key, value);
    }

    // Example 5: Complete pagination loop
    println!("\n📄 Example 5: Complete pagination through all items");
    println!("--------------------------------------------------");

    let mut current_key: Option<String> = None;
    let page_size = 8;
    let mut page_num = 1;
    let mut total_items = 0;

    loop {
        let mut props = ListProps::default().order(Order::Asc).limit(page_size);

        if let Some(ref key) = current_key {
            props = props.start_after_key(key);
        }

        let page = cache.list(props).unwrap();

        if page.is_empty() {
            break;
        }

        println!("Page {} ({} items):", page_num, page.len());
        for (key, _value) in &page {
            print!("{} ", key);
        }
        println!();

        total_items += page.len();
        current_key = Some(page.last().unwrap().0.clone());
        page_num += 1;
    }

    println!(
        "\n📊 Summary: {} pages, {} total items",
        page_num - 1,
        total_items
    );

    // Example 6: Edge case - limit of 0
    println!("\n📄 Example 6: Edge case - limit=0");
    println!("----------------------------------");

    let props = ListProps::default().limit(0);
    let empty_page = cache.list(props).unwrap();
    println!(
        "Page with limit=0: {} items (as expected)",
        empty_page.len()
    );
}

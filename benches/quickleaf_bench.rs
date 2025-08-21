use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use quickleaf::{Cache, Filter, ListProps, Order};
use std::sync::mpsc::channel;
use std::time::Duration;

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");

    for size in &[10, 100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut cache = Cache::new(size);
            let mut i = 0;
            b.iter(|| {
                cache.insert(format!("key{}", i), format!("value{}", i));
                i += 1;
                if i >= size {
                    i = 0;
                    cache.clear();
                }
            });
        });
    }

    group.finish();
}

fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");

    for size in &[10, 100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut cache = Cache::new(size);

            // Pre-populate the cache
            for i in 0..size {
                cache.insert(format!("key{}", i), format!("value{}", i));
            }

            let mut i = 0;
            b.iter(|| {
                black_box(cache.get(&format!("key{}", i)));
                i = (i + 1) % size;
            });
        });
    }

    group.finish();
}

fn bench_contains_key(c: &mut Criterion) {
    let mut group = c.benchmark_group("contains_key");

    for size in &[10, 100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut cache = Cache::new(size);

            // Pre-populate the cache
            for i in 0..size {
                cache.insert(format!("key{}", i), format!("value{}", i));
            }

            let mut i = 0;
            b.iter(|| {
                black_box(cache.contains_key(&format!("key{}", i)));
                i = (i + 1) % size;
            });
        });
    }

    group.finish();
}

fn bench_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove");

    group.bench_function("remove_and_reinsert", |b| {
        let mut cache = Cache::new(1000);

        // Pre-populate
        for i in 0..1000 {
            cache.insert(format!("key{}", i), format!("value{}", i));
        }

        let mut i = 0;
        b.iter(|| {
            let key = format!("key{}", i);
            cache.remove(&key).ok();
            cache.insert(key, format!("value{}", i));
            i = (i + 1) % 1000;
        });
    });

    group.finish();
}

fn bench_list_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_operations");

    // Benchmark listing with different filters
    group.bench_function("list_no_filter", |b| {
        let mut cache = Cache::new(1000);

        for i in 0..1000 {
            cache.insert(format!("item{:04}", i), i);
        }

        b.iter(|| {
            let mut props = ListProps::default().order(Order::Asc);
            props.limit = 100;
            black_box(cache.list(props).unwrap());
        });
    });

    group.bench_function("list_with_start_filter", |b| {
        let mut cache = Cache::new(1000);

        for i in 0..1000 {
            cache.insert(format!("item{:04}", i), i);
        }

        b.iter(|| {
            let mut props = ListProps::default()
                .order(Order::Asc)
                .filter(Filter::StartWith("item00".to_string()));
            props.limit = 50;
            black_box(cache.list(props).unwrap());
        });
    });

    group.bench_function("list_with_end_filter", |b| {
        let mut cache = Cache::new(1000);

        for i in 0..1000 {
            cache.insert(format!("item{:04}", i), i);
        }

        b.iter(|| {
            let mut props = ListProps::default()
                .order(Order::Desc)
                .filter(Filter::EndWith("99".to_string()));
            props.limit = 50;
            black_box(cache.list(props).unwrap());
        });
    });

    group.finish();
}

fn bench_lru_eviction(c: &mut Criterion) {
    c.bench_function("lru_eviction", |b| {
        let mut cache = Cache::new(100); // Small capacity to trigger evictions
        let mut i = 0;

        b.iter(|| {
            cache.insert(format!("key{}", i), format!("value{}", i));
            i += 1;
        });
    });
}

fn bench_ttl_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ttl_operations");

    group.bench_function("insert_with_ttl", |b| {
        let mut cache = Cache::new(1000);
        let mut i = 0;

        b.iter(|| {
            cache.insert_with_ttl(
                format!("ttl_key{}", i),
                format!("ttl_value{}", i),
                Duration::from_secs(60),
            );
            i = (i + 1) % 1000;
        });
    });

    group.bench_function("cleanup_expired", |b| {
        let mut cache = Cache::new(1000);

        // Insert items with very short TTL
        for i in 0..500 {
            cache.insert_with_ttl(
                format!("expired{}", i),
                format!("value{}", i),
                Duration::from_nanos(1), // Will expire immediately
            );
        }

        // Insert permanent items
        for i in 500..1000 {
            cache.insert(format!("permanent{}", i), format!("value{}", i));
        }

        b.iter(|| {
            black_box(cache.cleanup_expired());
        });
    });

    group.bench_function("get_with_expired_check", |b| {
        let mut cache = Cache::new(1000);

        // Mix of expired and valid items
        for i in 0..500 {
            cache.insert_with_ttl(
                format!("expired{}", i),
                format!("value{}", i),
                Duration::from_nanos(1),
            );
        }

        for i in 500..1000 {
            cache.insert(format!("valid{}", i), format!("value{}", i));
        }

        let mut i = 0;
        b.iter(|| {
            if i < 500 {
                black_box(cache.get(&format!("expired{}", i)));
            } else {
                black_box(cache.get(&format!("valid{}", i)));
            }
            i = (i + 1) % 1000;
        });
    });

    group.finish();
}

fn bench_event_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_system");

    group.bench_function("insert_with_events", |b| {
        let (tx, rx) = channel();
        let mut cache = Cache::with_sender(1000, tx);
        let mut i = 0;

        b.iter(|| {
            cache.insert(format!("event_key{}", i), format!("event_value{}", i));
            // Drain the receiver to avoid blocking
            while rx.try_recv().is_ok() {}
            i = (i + 1) % 1000;
        });
    });

    group.bench_function("operations_without_events", |b| {
        let mut cache = Cache::new(1000);
        let mut i = 0;

        b.iter(|| {
            cache.insert(format!("key{}", i), format!("value{}", i));
            i = (i + 1) % 1000;
        });
    });

    group.finish();
}

fn bench_mixed_operations(c: &mut Criterion) {
    c.bench_function("mixed_operations", |b| {
        let mut cache = Cache::new(1000);

        // Pre-populate
        for i in 0..500 {
            cache.insert(format!("key{}", i), format!("value{}", i));
        }

        let mut i = 0;
        b.iter(|| {
            match i % 4 {
                0 => {
                    cache.insert(format!("key{}", i + 500), format!("value{}", i + 500));
                }
                1 => {
                    black_box(cache.get(&format!("key{}", i % 500)));
                }
                2 => {
                    black_box(cache.contains_key(&format!("key{}", i % 500)));
                }
                3 => {
                    if i < 500 {
                        cache.remove(&format!("key{}", i)).ok();
                        cache.insert(format!("key{}", i), format!("value{}", i));
                    }
                }
                _ => unreachable!(),
            }
            i = (i + 1) % 2000;
        });
    });
}

fn bench_value_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_types");

    group.bench_function("insert_strings", |b| {
        let mut cache = Cache::new(1000);
        let mut i = 0;

        b.iter(|| {
            cache.insert(
                format!("key{}", i),
                format!("This is a longer string value {}", i),
            );
            i = (i + 1) % 1000;
        });
    });

    group.bench_function("insert_integers", |b| {
        let mut cache = Cache::new(1000);
        let mut i = 0;

        b.iter(|| {
            cache.insert(format!("key{}", i), i);
            i = (i + 1) % 1000;
        });
    });

    group.bench_function("insert_floats", |b| {
        let mut cache = Cache::new(1000);
        let mut i = 0;

        b.iter(|| {
            cache.insert(format!("key{}", i), i as f64 * 3.14159);
            i = (i + 1) % 1000;
        });
    });

    group.bench_function("insert_booleans", |b| {
        let mut cache = Cache::new(1000);
        let mut i = 0;

        b.iter(|| {
            cache.insert(format!("key{}", i), i % 2 == 0);
            i = (i + 1) % 1000;
        });
    });

    group.finish();
}

#[cfg(feature = "persist")]
fn bench_persistence(c: &mut Criterion) {
    use std::fs;

    let mut group = c.benchmark_group("persistence");

    group.bench_function("persist_insert", |b| {
        let db_path = "/tmp/bench_persist.db";
        let _ = fs::remove_file(db_path);

        let mut cache = Cache::with_persist(db_path, 1000).unwrap();
        let mut i = 0;

        b.iter(|| {
            cache.insert(format!("persist_key{}", i), format!("persist_value{}", i));
            i = (i + 1) % 1000;
        });

        let _ = fs::remove_file(db_path);
    });

    group.bench_function("persist_with_ttl", |b| {
        let db_path = "/tmp/bench_persist_ttl.db";
        let _ = fs::remove_file(db_path);

        let mut cache =
            Cache::with_persist_and_ttl(db_path, 1000, Duration::from_secs(3600)).unwrap();
        let mut i = 0;

        b.iter(|| {
            cache.insert(format!("ttl_key{}", i), format!("ttl_value{}", i));
            i = (i + 1) % 1000;
        });

        let _ = fs::remove_file(db_path);
    });

    group.bench_function("persist_load", |b| {
        let db_path = "/tmp/bench_persist_load.db";
        let _ = fs::remove_file(db_path);

        // Pre-populate database
        {
            let mut cache = Cache::with_persist(db_path, 1000).unwrap();
            for i in 0..1000 {
                cache.insert(format!("key{}", i), format!("value{}", i));
            }
            std::thread::sleep(Duration::from_millis(100)); // Wait for persistence
        }

        b.iter(|| {
            black_box(Cache::with_persist(db_path, 1000).unwrap());
        });

        let _ = fs::remove_file(db_path);
    });

    group.finish();
}

fn bench_capacity_limits(c: &mut Criterion) {
    let mut group = c.benchmark_group("capacity_limits");

    for capacity in &[10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("eviction_overhead", capacity),
            capacity,
            |b, &capacity| {
                let mut cache = Cache::new(capacity);
                let mut i = 0;

                // Pre-fill to capacity
                for j in 0..capacity {
                    cache.insert(format!("init{}", j), format!("value{}", j));
                }

                b.iter(|| {
                    // This will always trigger eviction
                    cache.insert(format!("overflow{}", i), format!("value{}", i));
                    i += 1;
                });
            },
        );
    }

    group.finish();
}

// Main benchmark groups
criterion_group!(
    benches,
    bench_insert,
    bench_get,
    bench_contains_key,
    bench_remove,
    bench_list_operations,
    bench_lru_eviction,
    bench_ttl_operations,
    bench_event_system,
    bench_mixed_operations,
    bench_value_types,
    bench_capacity_limits
);

// Add persistence benchmarks only when the feature is enabled
#[cfg(feature = "persist")]
criterion_group!(persist_benches, bench_persistence);

// Main entry point
#[cfg(not(feature = "persist"))]
criterion_main!(benches);

#[cfg(feature = "persist")]
criterion_main!(benches, persist_benches);

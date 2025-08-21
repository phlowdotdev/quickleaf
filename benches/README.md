# Quickleaf Benchmarks

This directory contains performance benchmarks for the Quickleaf cache implementation using Criterion.rs.

## Running Benchmarks

### Run all benchmarks:
```bash
cargo bench
```

### Run specific benchmark groups:
```bash
# Run only insert benchmarks
cargo bench insert

# Run only get benchmarks  
cargo bench get

# Run only list operations
cargo bench list_operations

# Run only TTL operations
cargo bench ttl_operations
```

### Run benchmarks with persistence feature:
```bash
cargo bench --features persist
```

### Generate HTML reports:
```bash
cargo bench
# Reports will be generated in target/criterion/
```

## Benchmark Groups

The benchmark suite includes the following test groups:

### Core Operations
- **insert**: Tests insertion performance with various cache sizes (10, 100, 1000, 10000)
- **get**: Tests retrieval performance from pre-populated caches
- **contains_key**: Tests key existence checking
- **remove**: Tests removal and reinsertion operations

### Advanced Features
- **list_operations**: Tests listing with filters and ordering
  - No filter
  - StartWith filter
  - EndWith filter
- **lru_eviction**: Tests LRU eviction overhead
- **ttl_operations**: Tests TTL-based features
  - Insert with TTL
  - Cleanup expired items
  - Get with expired check

### System Features
- **event_system**: Compares operations with and without event notifications
- **mixed_operations**: Tests realistic mixed workloads
- **value_types**: Tests different value types (strings, integers, floats, booleans)
- **capacity_limits**: Tests eviction overhead at different capacities

### Persistence (optional)
- **persist_insert**: Tests insertion with SQLite persistence
- **persist_with_ttl**: Tests persistence with TTL support
- **persist_load**: Tests loading from persisted database

## Interpreting Results

Criterion will provide:
- Median and mean execution times
- Standard deviation
- Throughput measurements
- Performance comparisons between runs

HTML reports include:
- Violin plots showing distribution
- Line charts showing performance trends
- Regression detection between runs

## Tips for Benchmarking

1. **Close unnecessary applications** to reduce system noise
2. **Run benchmarks multiple times** to ensure consistency
3. **Use release mode** (cargo bench automatically uses optimized builds)
4. **Check baseline** before making optimizations
5. **Save results** for comparison after changes

## Example Output

```
insert/10               time:   [195.32 ns 196.45 ns 197.71 ns]
insert/100              time:   [201.15 ns 202.89 ns 204.78 ns]
insert/1000             time:   [208.93 ns 210.12 ns 211.45 ns]
insert/10000            time:   [215.67 ns 217.23 ns 219.01 ns]

get/10                  time:   [45.123 ns 45.456 ns 45.812 ns]
get/100                 time:   [46.234 ns 46.567 ns 46.923 ns]
get/1000                time:   [47.345 ns 47.678 ns 48.034 ns]
get/10000               time:   [48.456 ns 48.789 ns 49.145 ns]
```

## Customizing Benchmarks

To add new benchmarks, edit `benches/quickleaf_bench.rs` and:

1. Create a new benchmark function
2. Add it to the appropriate `criterion_group!`
3. Run `cargo bench` to test

## Performance Targets

Based on the cache design, expected performance characteristics:
- O(1) insert and get operations
- Minimal overhead from event system
- TTL checks should be lazy (no performance impact when not expired)
- Persistence should use background threads (minimal impact on operations)

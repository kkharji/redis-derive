# Redis-Derive Benchmarks Guide

## 🚀 Quick Start

### Running All Benchmarks
```bash
cargo bench
```

### Running Specific Benchmarks
```bash
# Run only simple struct benchmarks
cargo bench simple

# Run only complex struct benchmarks  
cargo bench complex

# Run only enum benchmarks
cargo bench enum

# Run memory usage benchmarks
cargo bench memory
```

### Generate HTML Report
```bash
cargo bench -- --output-format html
```

## 📊 Benchmark Categories

### 1. **Simple Struct Performance**
Tests basic struct serialization/deserialization:
```rust
struct SimpleStruct {
    id: u64,
    name: String, 
    active: bool,
}
```
- `simple_to_redis_args`: Serialization speed
- `simple_from_redis_value`: Deserialization speed

### 2. **Complex Struct Performance**
Tests realistic complex structures:
```rust
struct ComplexStruct {
    id: u64,
    username: String,
    email: Option<String>,
    scores: Vec<f64>,
    metadata: HashMap<String, String>,
    tags: Vec<String>,
    settings: SubStruct,
    // ... more fields
}
```
- `complex_to_redis_args`: Complex serialization
- `complex_from_redis_value`: Complex deserialization

### 3. **Enum Performance** 
Tests enum transformations with `rename_all`:
```rust
#[redis(rename_all = "snake_case")]
enum Status { Active, Inactive, Pending }
```
- `enum_to_redis_args`: Enum serialization + case conversion
- `enum_from_redis_value`: Enum parsing + case conversion

### 4. **Round-Trip Performance**
Tests full serialize→deserialize cycles:
- `round_trip_simple`: Complete round-trip timing

### 5. **Memory & Scalability**
Tests performance with large datasets:
- `create_many_structs`: Memory allocation patterns
- `large_collections_to_redis_args`: Large collection handling

### 6. **Manual vs Derived Comparison**
Compares hand-written vs macro-generated code:
- `manual_simple_struct`: Hand-optimized implementation
- `derived_simple_struct`: Macro-generated implementation

## 📈 Expected Results

### Typical Performance Characteristics

**Simple Structs** (3 fields):
- Serialization: ~50-200 ns
- Deserialization: ~100-500 ns

**Complex Structs** (10+ fields, nested):  
- Serialization: ~500ns-2μs
- Deserialization: ~1-5μs

**Enums**:
- Serialization: ~10-50 ns
- Deserialization: ~50-200 ns

**Large Collections** (1000+ items):
- Serialization: ~10-100μs
- Deserialization: ~20-200μs

### Performance Tips

**🔥 Hot Paths**:
- Simple structs are fastest
- Enums with `rename_all` add ~10-20% overhead
- Nested structs multiply costs

**🐌 Slow Paths**:
- Large `Vec<T>` fields
- Many `String` fields (allocation heavy)
- Deep nesting (>3 levels)

## 🛠️ Optimization Insights

### What to Benchmark When Optimizing

1. **Before/After Changes**:
   ```bash
   # Before changes
   cargo bench > before.txt
   
   # After changes  
   cargo bench > after.txt
   
   # Compare
   diff before.txt after.txt
   ```

2. **Regression Testing**:
   ```bash
   # Save baseline
   cargo bench --save-baseline main
   
   # Test changes
   cargo bench --baseline main
   ```

3. **Profiling Integration**:
   ```bash
   # Profile with perf (Linux)
   cargo bench --bench redis_derive_bench -- --profile-time=5
   ```

### Benchmark Interpretation

**Good Performance Indicators**:
- ✅ Sub-microsecond simple struct operations
- ✅ Linear scaling with data size
- ✅ Minimal allocation overhead
- ✅ Comparable to manual implementations

**Performance Warning Signs**:
- ⚠️ >10μs for simple operations
- ⚠️ Exponential scaling
- ⚠️ Excessive memory allocation
- ⚠️ 5x+ slower than manual code

## 🔍 Custom Benchmarks

### Adding Your Own Benchmarks

1. **Create Test Struct**:
   ```rust
   #[derive(ToRedisArgs, FromRedisValue)]
   struct MyStruct {
       // Your fields
   }
   ```

2. **Add Benchmark Function**:
   ```rust
   fn bench_my_struct(c: &mut Criterion) {
       let my_data = MyStruct { /* ... */ };
       
       c.bench_function("my_struct_to_redis", |b| {
           b.iter(|| {
               let _args = ArgCollector::collect_args(black_box(&my_data));
           })
       });
   }
   ```

3. **Add to Criterion Group**:
   ```rust
   criterion_group!(
       benches,
       // ... existing benchmarks
       bench_my_struct
   );
   ```

### Real-World Scenarios

**Session Management**:
```rust
#[derive(ToRedisArgs, FromRedisValue)]
struct UserSession {
    user_id: u64,
    session_token: String,
    expires_at: i64,
    permissions: Vec<String>,
    metadata: HashMap<String, String>,
}
```

**Caching Layer**:
```rust  
#[derive(ToRedisArgs, FromRedisValue)]
struct CacheEntry<T> {
    key: String,
    value: T,
    ttl: u64,
    version: u32,
}
```

## 📊 Continuous Benchmarking

### CI/CD Integration

```yaml
# .github/workflows/benchmark.yml
- name: Run Benchmarks
  run: cargo bench --bench redis_derive_bench
  
- name: Store Benchmark Results  
  uses: benchmark-action/github-action-benchmark@v1
  with:
    tool: 'cargo'
    output-file-path: target/criterion/reports/index.html
```

### Performance Regression Detection

```bash
# Set performance thresholds
cargo bench -- --significance-level 0.05 --noise-threshold 0.02
```

The benchmarks provide comprehensive performance insights to help you optimize redis-derive usage and catch performance regressions early!
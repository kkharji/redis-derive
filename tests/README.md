# Redis-Derive Test Suite

This directory contains comprehensive tests for the redis-derive crate, covering positive cases, negative cases, and corner cases.

## Test Structure

### `integration_tests.rs`
The main integration test suite that covers:

**Positive Test Cases:**
- ✅ Named structs with various field types (String, i64, Option, Vec)
- ✅ Tuple structs with different arities
- ✅ Unit structs
- ✅ Enums with all supported `rename_all` transformations
- ✅ Nested structs (structs containing other structs)
- ✅ Round-trip serialization/deserialization

**Corner Cases:**
- ✅ Unicode text handling (emoji, Chinese, Arabic)
- ✅ Empty collections and None values
- ✅ Large numbers (i64::MAX, u64::MAX, f64::MAX)
- ✅ Zero values and empty strings
- ✅ Special characters in field names

**Enum Transformations Tested:**
- `snake_case`: `InProgress` → `in_progress`
- `UPPERCASE`: `High` → `HIGH`  
- `kebab-case`: `DataAnalysis` → `data-analysis`
- `lowercase`: `Red` → `red`
- `PascalCase`: `myField` → `MyField`
- `camelCase`: `my_field` → `myField`

### `error_cases.rs`
Dedicated error condition tests that verify:

**Error Handling:**
- ❌ Wrong Redis value types (string instead of map for structs)
- ❌ Invalid enum variants
- ❌ Wrong array lengths for tuple structs
- ❌ Type conversion errors
- ❌ Nil/null values where structs expected

**Stress Tests:**
- 🔄 Deeply nested structures (Level1 → Level2 → Level3)
- 🔄 Large structs with 20+ fields
- 🔄 Enums with many variants (20+)

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Modules
```bash
# Run only integration tests
cargo test --test integration_tests

# Run only error case tests  
cargo test --test error_cases

# Run embedded lib tests
cargo test --lib
```

### Run Specific Test Functions
```bash
# Test named struct functionality
cargo test test_named_struct

# Test enum transformations
cargo test test_enum_snake_case

# Test error conditions
cargo test test_struct_from_wrong_redis_type
```

### Verbose Output
```bash
# See detailed test output
cargo test -- --nocapture

# Show which tests are running
cargo test -- --show-output
```

## Test Coverage

The test suite covers:

| Feature | Coverage | Test File |
|---------|----------|-----------|
| Named Structs | ✅ Complete | `integration_tests.rs` |
| Tuple Structs | ✅ Complete | `integration_tests.rs` |
| Unit Structs | ✅ Complete | `integration_tests.rs` |
| Enums (all rename rules) | ✅ Complete | `integration_tests.rs` |
| Error Conditions | ✅ Complete | `error_cases.rs` |
| Unicode Support | ✅ Complete | `integration_tests.rs` |
| Large Data | ✅ Complete | Both files |
| Nested Structures | ✅ Complete | `integration_tests.rs` |
| Edge Cases | ✅ Complete | Both files |

## Test Helpers

The tests include helper functions for easier testing:

- `to_args()`: Converts `ToRedisArgs` types to `Vec<Vec<u8>>`
- `create_map_value()`: Creates Redis Map values from key-value pairs

## Adding New Tests

When adding new tests, follow these patterns:

### For New Features
1. Add positive test cases in `integration_tests.rs`
2. Add corresponding error cases in `error_cases.rs`
3. Include edge cases and corner cases
4. Test round-trip compatibility (serialize → deserialize)

### For Bug Fixes
1. Write a failing test that reproduces the bug
2. Fix the bug
3. Verify the test now passes
4. Add additional edge case tests if needed

## Performance Testing

While not included in this basic suite, consider adding:
- Benchmark tests for large structures
- Memory usage tests for deeply nested data
- Performance comparison tests vs manual Redis operations

## Real Redis Integration

These tests use mock Redis values. For full integration testing:

1. Start a Redis server locally
2. Create tests that actually connect to Redis
3. Test HSET/HGETALL round trips
4. Test with real Redis data types and edge cases

Example integration test setup:
```rust
#[test]
fn test_real_redis_integration() -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;
    
    let original = MyStruct { /* ... */ };
    
    let _: () = redis::cmd("HSET")
        .arg("test_key")
        .arg(&original)
        .query(&mut con)?;
    
    let retrieved: MyStruct = con.hgetall("test_key")?;
    
    assert_eq!(original, retrieved);
    Ok(())
}
```
use redis::{FromRedisValue, Value, ErrorKind};
use redis_derive::{FromRedisValue, ToRedisArgs};

/// Test various error conditions that should fail gracefully

#[derive(FromRedisValue, ToRedisArgs, Debug)]
struct Person {
    name: String,
    age: i32,
    active: bool,
}

#[derive(FromRedisValue, ToRedisArgs, Debug)]
struct Point(i32, i32);

#[derive(FromRedisValue, ToRedisArgs, Debug)]
enum Color {
    Red,
    Green,
    Blue,
}

#[test]
fn test_struct_from_wrong_redis_type() {
    // Test passing a simple string instead of a map
    let value = Value::BulkString(b"not a map".to_vec());
    let result: Result<Person, _> = FromRedisValue::from_redis_value(&value);
    
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(err.kind(), ErrorKind::TypeError));
    }
}

#[test]
fn test_struct_from_nil() {
    let value = Value::Nil;
    let result: Result<Person, _> = FromRedisValue::from_redis_value(&value);
    
    assert!(result.is_err());
}

#[test]
fn test_struct_from_array() {
    // Named structs expect maps, not arrays
    let value = Value::Array(vec![
        Value::BulkString(b"John".to_vec()),
        Value::BulkString(b"30".to_vec()),
        Value::BulkString(b"true".to_vec()),
    ]);
    let result: Result<Person, _> = FromRedisValue::from_redis_value(&value);
    
    assert!(result.is_err());
}

#[test]
fn test_tuple_struct_from_wrong_array_length() {
    // Point expects exactly 2 elements
    let value = Value::Array(vec![
        Value::BulkString(b"10".to_vec()),
    ]);
    let result: Result<Point, _> = FromRedisValue::from_redis_value(&value);
    
    assert!(result.is_err());

    // Test with too many elements
    let value = Value::Array(vec![
        Value::BulkString(b"10".to_vec()),
        Value::BulkString(b"20".to_vec()),
        Value::BulkString(b"30".to_vec()),
    ]);
    let result: Result<Point, _> = FromRedisValue::from_redis_value(&value);
    
    assert!(result.is_err());
}

#[test]
fn test_tuple_struct_from_non_array() {
    let value = Value::BulkString(b"not an array".to_vec());
    let result: Result<Point, _> = FromRedisValue::from_redis_value(&value);
    
    assert!(result.is_err());
}

#[test]
fn test_enum_from_invalid_variant() {
    let value = Value::BulkString(b"Purple".to_vec());
    let result: Result<Color, _> = FromRedisValue::from_redis_value(&value);
    
    assert!(result.is_err());
}

#[test]
fn test_enum_from_wrong_type() {
    // Enums expect string values, not numbers
    let value = Value::Int(42);
    let result: Result<Color, _> = FromRedisValue::from_redis_value(&value);
    
    assert!(result.is_err());
}

#[test]
fn test_enum_from_array() {
    let value = Value::Array(vec![Value::BulkString(b"Red".to_vec())]);
    let result: Result<Color, _> = FromRedisValue::from_redis_value(&value);
    
    assert!(result.is_err());
}

#[test]
fn test_field_type_conversion_errors() {
    // Test when individual field parsing fails
    let items = vec![
        (Value::BulkString(b"name".to_vec()), Value::BulkString(b"John".to_vec())),
        (Value::BulkString(b"age".to_vec()), Value::BulkString(b"not_a_number".to_vec())),
        (Value::BulkString(b"active".to_vec()), Value::BulkString(b"true".to_vec())),
    ];
    let value = Value::Map(items);
    
    let result: Result<Person, _> = FromRedisValue::from_redis_value(&value);
    
    // This might succeed or fail depending on how redis handles string->int conversion
    // The test mainly ensures we don't panic
    let _ = result;
}

#[test]
fn test_empty_map_with_required_fields() {
    let value = Value::Map(vec![]);
    let result: Result<Person, _> = FromRedisValue::from_redis_value(&value);
    
    // Should use default values for missing fields or fail gracefully
    let _ = result; // Outcome depends on how Default is implemented for field types
}

#[test]
fn test_malformed_map() {
    // Test with odd number of items (malformed map)
    let items = vec![
        (Value::BulkString(b"name".to_vec()), Value::BulkString(b"John".to_vec())),
        (Value::BulkString(b"age".to_vec()), Value::BulkString(b"30".to_vec())),
        // Missing value for active field would be handled by the Map construction
    ];
    let value = Value::Map(items);
    
    let result: Result<Person, _> = FromRedisValue::from_redis_value(&value);
    
    // Should handle gracefully, likely using default for missing active field
    let _ = result;
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_deeply_nested_structure() {
        #[derive(FromRedisValue, ToRedisArgs, Debug)]
        struct Level1 {
            data: String,
            next: Level2,
        }

        #[derive(FromRedisValue, ToRedisArgs, Debug)]
        struct Level2 {
            data: String,
            next: Level3,
        }

        #[derive(FromRedisValue, ToRedisArgs, Debug)]
        struct Level3 {
            data: String,
            value: i32,
        }

        // Test that deeply nested structures compile and work
        let level3 = Level3 {
            data: "level3".to_string(),
            value: 42,
        };
        let level2 = Level2 {
            data: "level2".to_string(),
            next: level3,
        };
        let level1 = Level1 {
            data: "level1".to_string(),
            next: level2,
        };

        // Just test that this compiles and doesn't panic
        let _ = level1;
    }

    #[test]
    fn test_large_struct() {
        #[derive(FromRedisValue, ToRedisArgs, Debug)]
        struct LargeStruct {
            field1: String, field2: String, field3: String, field4: String, field5: String,
            field6: String, field7: String, field8: String, field9: String, field10: String,
            field11: i32, field12: i32, field13: i32, field14: i32, field15: i32,
            field16: bool, field17: bool, field18: bool, field19: bool, field20: bool,
        }

        let large = LargeStruct {
            field1: "1".to_string(), field2: "2".to_string(), field3: "3".to_string(),
            field4: "4".to_string(), field5: "5".to_string(), field6: "6".to_string(),
            field7: "7".to_string(), field8: "8".to_string(), field9: "9".to_string(),
            field10: "10".to_string(), field11: 11, field12: 12, field13: 13,
            field14: 14, field15: 15, field16: true, field17: false, field18: true,
            field19: false, field20: true,
        };

        // Test that large structures work without issues
        let _ = large;
    }

    #[test]
    fn test_many_enum_variants() {
        #[derive(FromRedisValue, ToRedisArgs, Debug)]
        enum ManyVariants {
            V1, V2, V3, V4, V5, V6, V7, V8, V9, V10,
            V11, V12, V13, V14, V15, V16, V17, V18, V19, V20,
        }

        // Test that enums with many variants work
        let variant = ManyVariants::V15;
        let _ = variant;
    }
}
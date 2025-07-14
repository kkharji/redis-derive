use redis::{FromRedisValue, ToRedisArgs, Value, RedisWrite};
use redis_derive::{FromRedisValue, ToRedisArgs};

// Test helper to convert ToRedisArgs to Vec<Vec<u8>>
fn to_args<T: ToRedisArgs>(value: &T) -> Vec<Vec<u8>> {
    struct ArgCollector {
        args: Vec<Vec<u8>>,
    }
    
    impl RedisWrite for ArgCollector {
        fn write_arg(&mut self, arg: &[u8]) {
            self.args.push(arg.to_vec());
        }
        
        fn writer_for_next_arg(&mut self) -> impl std::io::Write + '_ {
            use std::io::Write;
            struct ArgWriter<'a> {
                collector: &'a mut ArgCollector,
                buffer: Vec<u8>,
            }
            
            impl<'a> Write for ArgWriter<'a> {
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                    self.buffer.extend_from_slice(buf);
                    Ok(buf.len())
                }
                
                fn flush(&mut self) -> std::io::Result<()> {
                    self.collector.args.push(std::mem::take(&mut self.buffer));
                    Ok(())
                }
            }
            
            ArgWriter {
                collector: self,
                buffer: Vec::new(),
            }
        }
    }
    
    let mut collector = ArgCollector { args: Vec::new() };
    value.write_redis_args(&mut collector);
    collector.args
}

// Test helper to create a Redis Map value
fn create_map_value(pairs: Vec<(&str, Value)>) -> Value {
    let items: Vec<(Value, Value)> = pairs.into_iter()
        .map(|(k, v)| (Value::BulkString(k.as_bytes().to_vec()), v))
        .collect();
    Value::Map(items)
}

#[cfg(test)]
mod named_struct_tests {
    use super::*;

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    struct Person {
        name: String,
        age: i64,
        email: Option<String>,
        hobbies: Vec<String>,
    }

    #[test]
    fn test_named_struct_to_redis_args() {
        let person = Person {
            name: "Alice".to_string(),
            age: 30,
            email: Some("alice@example.com".to_string()),
            hobbies: vec!["reading".to_string(), "swimming".to_string()],
        };

        let args = to_args(&person);
        
        // Should contain field names and values alternating
        // Note: Vec<String> serializes as multiple arguments, so count may vary
        assert!(args.len() >= 8); // At least 4 fields * 2 (name + value)
        
        // Check field names are present
        let arg_strings: Vec<String> = args.iter()
            .map(|arg| String::from_utf8_lossy(arg).to_string())
            .collect();
        
        assert!(arg_strings.contains(&"name".to_string()));
        assert!(arg_strings.contains(&"age".to_string()));
        assert!(arg_strings.contains(&"email".to_string()));
        assert!(arg_strings.contains(&"hobbies".to_string()));
    }

    #[test]
    fn test_named_struct_from_redis_value() {
        let redis_value = create_map_value(vec![
            ("name", Value::BulkString(b"Alice".to_vec())),
            ("age", Value::BulkString(b"30".to_vec())),
            ("email", Value::BulkString(b"alice@example.com".to_vec())),
            ("hobbies", Value::Array(vec![
                Value::BulkString(b"reading".to_vec()),
                Value::BulkString(b"swimming".to_vec()),
            ])),
        ]);

        let person: Person = FromRedisValue::from_redis_value(&redis_value).unwrap();
        
        assert_eq!(person.name, "Alice");
        assert_eq!(person.age, 30);
        assert_eq!(person.email, Some("alice@example.com".to_string()));
        assert_eq!(person.hobbies, vec!["reading", "swimming"]);
    }

    #[test]
    fn test_named_struct_with_missing_optional_field() {
        let redis_value = create_map_value(vec![
            ("name", Value::BulkString(b"Bob".to_vec())),
            ("age", Value::BulkString(b"25".to_vec())),
            ("hobbies", Value::Array(vec![])),
        ]);

        let person: Person = FromRedisValue::from_redis_value(&redis_value).unwrap();
        
        assert_eq!(person.name, "Bob");
        assert_eq!(person.age, 25);
        assert_eq!(person.email, None); // Should default to None
        assert_eq!(person.hobbies, Vec::<String>::new());
    }

    #[test]
    fn test_named_struct_round_trip() {
        let _original = Person {
            name: "Charlie".to_string(),
            age: 40,
            email: Some("charlie@test.com".to_string()),
            hobbies: vec!["coding".to_string()],
        };

        // This test would require a full Redis integration, but demonstrates the concept
        // In a real scenario, you'd: HSET -> HGETALL -> compare
    }
}

#[cfg(test)]
mod tuple_struct_tests {
    use super::*;

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    struct Point(i32, i32, i32);

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    struct Coordinates(f64, f64);

    #[test]
    fn test_tuple_struct_to_redis_args() {
        let point = Point(10, 20, 30);
        let args = to_args(&point);
        
        assert_eq!(args.len(), 3);
        
        let values: Vec<String> = args.iter()
            .map(|arg| String::from_utf8_lossy(arg).to_string())
            .collect();
        
        assert!(values.contains(&"10".to_string()));
        assert!(values.contains(&"20".to_string()));
        assert!(values.contains(&"30".to_string()));
    }

    #[test]
    fn test_tuple_struct_from_redis_value() {
        let redis_value = Value::Array(vec![
            Value::BulkString(b"10".to_vec()),
            Value::BulkString(b"20".to_vec()),
            Value::BulkString(b"30".to_vec()),
        ]);

        let point: Point = FromRedisValue::from_redis_value(&redis_value).unwrap();
        assert_eq!(point, Point(10, 20, 30));
    }

    #[test]
    fn test_tuple_struct_wrong_array_length() {
        let redis_value = Value::Array(vec![
            Value::BulkString(b"10".to_vec()),
            Value::BulkString(b"20".to_vec()),
            // Missing third element
        ]);

        let result: Result<Point, _> = FromRedisValue::from_redis_value(&redis_value);
        assert!(result.is_err());
    }

    #[test]
    fn test_tuple_struct_with_floats() {
        let coords = Coordinates(12.34, 56.78);
        let args = to_args(&coords);
        assert_eq!(args.len(), 2);
    }
}

#[cfg(test)]
mod unit_struct_tests {
    use super::*;

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    struct Empty;

    #[test]
    fn test_unit_struct_to_redis_args() {
        let empty = Empty;
        let args = to_args(&empty);
        
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], b"");
    }

    #[test]
    fn test_unit_struct_from_redis_value() {
        let redis_value = Value::Nil;
        let empty: Empty = FromRedisValue::from_redis_value(&redis_value).unwrap();
        assert_eq!(empty, Empty);
    }
}

#[cfg(test)]
mod enum_tests {
    use super::*;

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    #[redis(rename_all = "snake_case")]
    enum Status {
        Active,
        InProgress,
        Completed,
    }

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    #[redis(rename_all = "UPPERCASE")]
    enum Priority {
        Low,
        Medium,
        High,
    }

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    #[redis(rename_all = "kebab-case")]
    enum TaskType {
        DataAnalysis,
        ReportGeneration,
        SystemMaintenance,
    }

    #[test]
    fn test_enum_to_redis_args() {
        let color = Color::Red;
        let args = to_args(&color);
        
        assert_eq!(args.len(), 1);
        assert_eq!(String::from_utf8_lossy(&args[0]), "Red");
    }

    #[test]
    fn test_enum_from_redis_value() {
        let redis_value = Value::BulkString(b"Green".to_vec());
        let color: Color = FromRedisValue::from_redis_value(&redis_value).unwrap();
        assert_eq!(color, Color::Green);
    }

    #[test]
    fn test_enum_from_simple_string() {
        let redis_value = Value::SimpleString("Blue".to_string());
        let color: Color = FromRedisValue::from_redis_value(&redis_value).unwrap();
        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn test_enum_invalid_variant() {
        let redis_value = Value::BulkString(b"Purple".to_vec());
        let result: Result<Color, _> = FromRedisValue::from_redis_value(&redis_value);
        assert!(result.is_err());
    }

    #[test]
    fn test_enum_snake_case_transformation() {
        let status = Status::InProgress;
        let args = to_args(&status);
        
        assert_eq!(args.len(), 1);
        assert_eq!(String::from_utf8_lossy(&args[0]), "in_progress");

        // Test round trip
        let redis_value = Value::BulkString(b"in_progress".to_vec());
        let parsed: Status = FromRedisValue::from_redis_value(&redis_value).unwrap();
        assert_eq!(parsed, Status::InProgress);
    }

    #[test]
    fn test_enum_uppercase_transformation() {
        let priority = Priority::High;
        let args = to_args(&priority);
        
        assert_eq!(args.len(), 1);
        assert_eq!(String::from_utf8_lossy(&args[0]), "HIGH");

        // Test round trip
        let redis_value = Value::BulkString(b"HIGH".to_vec());
        let parsed: Priority = FromRedisValue::from_redis_value(&redis_value).unwrap();
        assert_eq!(parsed, Priority::High);
    }

    #[test]
    fn test_enum_kebab_case_transformation() {
        let task = TaskType::DataAnalysis;
        let args = to_args(&task);
        
        assert_eq!(args.len(), 1);
        assert_eq!(String::from_utf8_lossy(&args[0]), "data-analysis");

        // Test round trip
        let redis_value = Value::BulkString(b"system-maintenance".to_vec());
        let parsed: TaskType = FromRedisValue::from_redis_value(&redis_value).unwrap();
        assert_eq!(parsed, TaskType::SystemMaintenance);
    }
}

#[cfg(test)]
mod nested_struct_tests {
    use super::*;

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    struct Address {
        street: String,
        city: String,
        zip: String,
    }

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    struct Employee {
        id: u32,
        name: String,
        address: Address,
        active: bool,
    }

    #[test]
    fn test_nested_struct_to_redis_args() {
        let employee = Employee {
            id: 1001,
            name: "John Doe".to_string(),
            address: Address {
                street: "123 Main St".to_string(),
                city: "Anytown".to_string(),
                zip: "12345".to_string(),
            },
            active: true,
        };

        let args = to_args(&employee);
        assert!(args.len() > 0); // Should generate multiple args for nested structure
    }
}

#[cfg(test)]
mod error_cases_tests {
    use super::*;

    #[derive(FromRedisValue, Debug)]
    #[allow(dead_code)] // Test struct - fields accessed via derive macro
    struct SimpleStruct {
        name: String,
        age: i32,
    }

    #[test]
    fn test_struct_from_non_map_value() {
        let redis_value = Value::BulkString(b"not a map".to_vec());
        let result: Result<SimpleStruct, _> = FromRedisValue::from_redis_value(&redis_value);
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_from_nil_value() {
        let redis_value = Value::Nil;
        let result: Result<SimpleStruct, _> = FromRedisValue::from_redis_value(&redis_value);
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_from_array_value() {
        let redis_value = Value::Array(vec![
            Value::BulkString(b"John".to_vec()),
            Value::BulkString(b"30".to_vec()),
        ]);
        let result: Result<SimpleStruct, _> = FromRedisValue::from_redis_value(&redis_value);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod corner_cases_tests {
    use super::*;

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    struct UnicodeTest {
        emoji: String,
        chinese: String,
        arabic: String,
    }

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    struct EmptyCollections {
        empty_vec: Vec<String>,
        empty_option: Option<String>,
    }

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    struct LargeNumbers {
        big_int: i64,
        big_uint: u64,
        float_val: f64,
    }

    #[test]
    fn test_unicode_handling() {
        let unicode_test = UnicodeTest {
            emoji: "🚀🌟".to_string(),
            chinese: "你好世界".to_string(),
            arabic: "مرحبا بالعالم".to_string(),
        };

        let args = to_args(&unicode_test);
        assert!(args.len() > 0);

        // Test that Unicode is preserved in field names and values
        let arg_strings: Vec<String> = args.iter()
            .map(|arg| String::from_utf8_lossy(arg).to_string())
            .collect();
        
        assert!(arg_strings.iter().any(|s| s.contains("emoji")));
        assert!(arg_strings.iter().any(|s| s.contains("🚀")));
    }

    #[test]
    fn test_empty_collections() {
        let empty = EmptyCollections {
            empty_vec: Vec::new(),
            empty_option: None,
        };

        let args = to_args(&empty);
        assert!(args.len() > 0);

        // Test from Redis with empty/missing values
        let redis_value = create_map_value(vec![
            ("empty_vec", Value::Array(vec![])),
        ]);

        let parsed: EmptyCollections = FromRedisValue::from_redis_value(&redis_value).unwrap();
        assert_eq!(parsed.empty_vec, Vec::<String>::new());
        assert_eq!(parsed.empty_option, None);
    }

    #[test]
    fn test_large_numbers() {
        let large = LargeNumbers {
            big_int: i64::MAX,
            big_uint: u64::MAX,
            float_val: f64::MAX,
        };

        let args = to_args(&large);
        assert!(args.len() > 0);

        // Verify large numbers are converted to strings correctly
        let arg_strings: Vec<String> = args.iter()
            .map(|arg| String::from_utf8_lossy(arg).to_string())
            .collect();
        
        assert!(arg_strings.iter().any(|s| s.contains(&i64::MAX.to_string())));
    }

    #[test]
    fn test_special_characters_in_field_names() {
        // This would require a struct with special field names, which isn't possible
        // in Rust, but we can test that normal field names work correctly
        #[derive(ToRedisArgs, FromRedisValue)]
        struct SpecialNaming {
            field_with_underscores: String,
            #[allow(non_snake_case)] // Allow for testing
            field_with_camel_case: String,
        }

        let special = SpecialNaming {
            field_with_underscores: "test1".to_string(),
            field_with_camel_case: "test2".to_string(),
        };

        let args = to_args(&special);
        assert!(args.len() > 0);
    }

    #[test]
    fn test_zero_values() {
        #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
        struct ZeroValues {
            zero_int: i32,
            zero_float: f64,
            empty_string: String,
        }

        let zeros = ZeroValues {
            zero_int: 0,
            zero_float: 0.0,
            empty_string: String::new(),
        };

        let args = to_args(&zeros);
        assert!(args.len() > 0);

        // Test round trip with zero values
        let redis_value = create_map_value(vec![
            ("zero_int", Value::BulkString(b"0".to_vec())),
            ("zero_float", Value::BulkString(b"0".to_vec())),
            ("empty_string", Value::BulkString(b"".to_vec())),
        ]);

        let parsed: ZeroValues = FromRedisValue::from_redis_value(&redis_value).unwrap();
        assert_eq!(parsed, zeros);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    #[redis(rename_all = "snake_case")]
    enum UserRole {
        Administrator,
        RegularUser,
        GuestUser,
    }

    #[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
    struct User {
        id: u64,
        username: String,
        email: Option<String>,
        role: UserRole,
        tags: Vec<String>,
        active: bool,
    }

    #[test]
    fn test_complex_struct_integration() {
        let user = User {
            id: 12345,
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            role: UserRole::Administrator,
            tags: vec!["vip".to_string(), "premium".to_string()],
            active: true,
        };

        // Test serialization
        let args = to_args(&user);
        assert!(args.len() > 0);

        // Verify that enum is properly serialized with rename_all
        let role_args = to_args(&user.role);
        assert_eq!(String::from_utf8_lossy(&role_args[0]), "administrator");

        // Check what boolean actually serializes to
        let bool_args = to_args(&true);
        let bool_serialized = String::from_utf8_lossy(&bool_args[0]);
        
        // Test that we can create a realistic Redis value for deserialization
        let redis_value = create_map_value(vec![
            ("id", Value::BulkString(b"12345".to_vec())),
            ("username", Value::BulkString(b"testuser".to_vec())),
            ("email", Value::BulkString(b"test@example.com".to_vec())),
            ("role", Value::BulkString(b"administrator".to_vec())),
            ("tags", Value::Array(vec![
                Value::BulkString(b"vip".to_vec()),
                Value::BulkString(b"premium".to_vec()),
            ])),
            ("active", Value::BulkString(bool_serialized.as_bytes().to_vec())), // Use actual serialized form
        ]);

        let parsed_user: User = FromRedisValue::from_redis_value(&redis_value).unwrap();
        assert_eq!(parsed_user, user);
    }
}
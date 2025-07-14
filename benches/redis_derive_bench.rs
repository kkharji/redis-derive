// Benchmark tests for redis-derive performance
// Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use redis::{FromRedisValue, ToRedisArgs, Value, RedisWrite};
use redis_derive::{FromRedisValue, ToRedisArgs};
use std::collections::HashMap;

// Test structures for benchmarking
#[derive(ToRedisArgs, FromRedisValue, Debug, Clone)]
struct SimpleStruct {
    id: u64,
    name: String,
    active: bool,
}

#[derive(ToRedisArgs, FromRedisValue, Debug, Clone)]
struct ComplexStruct {
    id: u64,
    username: String,
    email: Option<String>,
    scores: Vec<f64>,
    metadata: HashMap<String, String>,
    tags: Vec<String>,
    created_at: String,
    updated_at: String,
    settings: SubStruct,
    active: bool,
}

#[derive(ToRedisArgs, FromRedisValue, Debug, Clone)]
struct SubStruct {
    theme: String,
    language: String,
    notifications: bool,
}

#[derive(ToRedisArgs, FromRedisValue, Debug, Clone, Default)]
#[redis(rename_all = "snake_case")]
enum Status {
    #[default]
    Active,
    Inactive,
    Pending,
    Suspended,
}

// Helper to collect ToRedisArgs output
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

impl ArgCollector {
    fn new() -> Self {
        Self { args: Vec::new() }
    }
    
    fn collect_args<T: ToRedisArgs>(value: &T) -> Vec<Vec<u8>> {
        let mut collector = Self::new();
        value.write_redis_args(&mut collector);
        collector.args
    }
}

// Helper to create test data
fn create_simple_struct() -> SimpleStruct {
    SimpleStruct {
        id: 12345,
        name: "Test User".to_string(),
        active: true,
    }
}

fn create_complex_struct() -> ComplexStruct {
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());
    metadata.insert("key2".to_string(), "value2".to_string());
    metadata.insert("key3".to_string(), "value3".to_string());
    
    ComplexStruct {
        id: 67890,
        username: "complex_user".to_string(),
        email: Some("user@example.com".to_string()),
        scores: vec![95.5, 87.2, 92.8, 89.1, 94.3],
        metadata,
        tags: vec![
            "premium".to_string(),
            "verified".to_string(),
            "active".to_string(),
        ],
        created_at: "2023-01-01T00:00:00Z".to_string(),
        updated_at: "2023-12-31T23:59:59Z".to_string(),
        settings: SubStruct {
            theme: "dark".to_string(),
            language: "en-US".to_string(),
            notifications: true,
        },
        active: true,
    }
}

fn create_redis_map_value(pairs: Vec<(&str, Value)>) -> Value {
    let items: Vec<(Value, Value)> = pairs.into_iter()
        .map(|(k, v)| (Value::BulkString(k.as_bytes().to_vec()), v))
        .collect();
    Value::Map(items)
}

fn create_simple_redis_value() -> Value {
    create_redis_map_value(vec![
        ("id", Value::BulkString(b"12345".to_vec())),
        ("name", Value::BulkString(b"Test User".to_vec())),
        ("active", Value::BulkString(b"true".to_vec())),
    ])
}

fn create_complex_redis_value() -> Value {
    create_redis_map_value(vec![
        ("id", Value::BulkString(b"67890".to_vec())),
        ("username", Value::BulkString(b"complex_user".to_vec())),
        ("email", Value::BulkString(b"user@example.com".to_vec())),
        ("scores", Value::Array(vec![
            Value::BulkString(b"95.5".to_vec()),
            Value::BulkString(b"87.2".to_vec()),
            Value::BulkString(b"92.8".to_vec()),
            Value::BulkString(b"89.1".to_vec()),
            Value::BulkString(b"94.3".to_vec()),
        ])),
        ("tags", Value::Array(vec![
            Value::BulkString(b"premium".to_vec()),
            Value::BulkString(b"verified".to_vec()),
            Value::BulkString(b"active".to_vec()),
        ])),
        ("created_at", Value::BulkString(b"2023-01-01T00:00:00Z".to_vec())),
        ("updated_at", Value::BulkString(b"2023-12-31T23:59:59Z".to_vec())),
        ("active", Value::BulkString(b"true".to_vec())),
        ("settings", create_redis_map_value(vec![
            ("theme", Value::BulkString(b"dark".to_vec())),
            ("language", Value::BulkString(b"en-US".to_vec())),
            ("notifications", Value::BulkString(b"true".to_vec())),
        ])),
        ("metadata", create_redis_map_value(vec![
            ("key1", Value::BulkString(b"value1".to_vec())),
            ("key2", Value::BulkString(b"value2".to_vec())),
            ("key3", Value::BulkString(b"value3".to_vec())),
        ])),
    ])
}

// Benchmark functions
fn bench_simple_to_redis_args(c: &mut Criterion) {
    let simple = create_simple_struct();
    
    c.bench_function("simple_to_redis_args", |b| {
        b.iter(|| {
            let _args = ArgCollector::collect_args(black_box(&simple));
        })
    });
}

fn bench_simple_from_redis_value(c: &mut Criterion) {
    let redis_value = create_simple_redis_value();
    
    c.bench_function("simple_from_redis_value", |b| {
        b.iter(|| {
            let _struct: Result<SimpleStruct, _> = FromRedisValue::from_redis_value(black_box(&redis_value));
        })
    });
}

fn bench_complex_to_redis_args(c: &mut Criterion) {
    let complex = create_complex_struct();
    
    c.bench_function("complex_to_redis_args", |b| {
        b.iter(|| {
            let _args = ArgCollector::collect_args(black_box(&complex));
        })
    });
}

fn bench_complex_from_redis_value(c: &mut Criterion) {
    let redis_value = create_complex_redis_value();
    
    c.bench_function("complex_from_redis_value", |b| {
        b.iter(|| {
            let _struct: Result<ComplexStruct, _> = FromRedisValue::from_redis_value(black_box(&redis_value));
        })
    });
}

fn bench_enum_to_redis_args(c: &mut Criterion) {
    let status = Status::Active;
    
    c.bench_function("enum_to_redis_args", |b| {
        b.iter(|| {
            let _args = ArgCollector::collect_args(black_box(&status));
        })
    });
}

fn bench_enum_from_redis_value(c: &mut Criterion) {
    let redis_value = Value::BulkString(b"active".to_vec());
    
    c.bench_function("enum_from_redis_value", |b| {
        b.iter(|| {
            let _enum: Result<Status, _> = FromRedisValue::from_redis_value(black_box(&redis_value));
        })
    });
}

fn bench_round_trip_simple(c: &mut Criterion) {
    let simple = create_simple_struct();
    let redis_value = create_simple_redis_value();
    
    c.bench_function("round_trip_simple", |b| {
        b.iter(|| {
            // Serialize
            let _args = ArgCollector::collect_args(black_box(&simple));
            // Deserialize  
            let _struct: Result<SimpleStruct, _> = FromRedisValue::from_redis_value(black_box(&redis_value));
        })
    });
}

fn bench_memory_usage(c: &mut Criterion) {
    c.bench_function("create_many_structs", |b| {
        b.iter(|| {
            let structs: Vec<SimpleStruct> = (0..1000)
                .map(|i| SimpleStruct {
                    id: i,
                    name: format!("User {}", i),
                    active: i % 2 == 0,
                })
                .collect();
            black_box(structs);
        })
    });
}

fn bench_large_collections(c: &mut Criterion) {
    let large_struct = ComplexStruct {
        id: 1,
        username: "large_test".to_string(),
        email: Some("test@example.com".to_string()),
        scores: (0..1000).map(|i| i as f64).collect(),
        metadata: (0..100).map(|i| (format!("key{}", i), format!("value{}", i))).collect(),
        tags: (0..50).map(|i| format!("tag{}", i)).collect(),
        created_at: "2023-01-01T00:00:00Z".to_string(),
        updated_at: "2023-12-31T23:59:59Z".to_string(),
        settings: SubStruct {
            theme: "dark".to_string(),
            language: "en-US".to_string(),
            notifications: true,
        },
        active: true,
    };
    
    c.bench_function("large_collections_to_redis_args", |b| {
        b.iter(|| {
            let _args = ArgCollector::collect_args(black_box(&large_struct));
        })
    });
}

fn bench_comparison_manual_vs_derive(c: &mut Criterion) {
    let simple = create_simple_struct();
    
    // Manual implementation for comparison
    struct ManualRedisArgs {
        args: Vec<Vec<u8>>,
    }
    
    impl RedisWrite for ManualRedisArgs {
        fn write_arg(&mut self, arg: &[u8]) {
            self.args.push(arg.to_vec());
        }
        
        fn writer_for_next_arg(&mut self) -> impl std::io::Write + '_ {
            use std::io::Write;
            struct ArgWriter<'a> {
                collector: &'a mut ManualRedisArgs,
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
    
    c.bench_function("manual_simple_struct", |b| {
        b.iter(|| {
            let mut collector = ManualRedisArgs { args: Vec::new() };
            let s = black_box(&simple);
            collector.write_arg(b"id");
            collector.write_arg(s.id.to_string().as_bytes());
            collector.write_arg(b"name");
            collector.write_arg(s.name.as_bytes());
            collector.write_arg(b"active");
            collector.write_arg(s.active.to_string().as_bytes());
            black_box(collector.args);
        })
    });
    
    c.bench_function("derived_simple_struct", |b| {
        b.iter(|| {
            let _args = ArgCollector::collect_args(black_box(&simple));
        })
    });
}

// Group all benchmarks
criterion_group!(
    benches,
    bench_simple_to_redis_args,
    bench_simple_from_redis_value,
    bench_complex_to_redis_args,
    bench_complex_from_redis_value,
    bench_enum_to_redis_args,
    bench_enum_from_redis_value,
    bench_round_trip_simple,
    bench_memory_usage,
    bench_large_collections,
    bench_comparison_manual_vs_derive
);

criterion_main!(benches);
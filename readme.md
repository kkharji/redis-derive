# redis-derive

## redis-derive

This crate implements the `FromRedisValue`(redis::FromRedisValue) and `ToRedisArgs`(redis::ToRedisArgs) traits
from `redis-rs`(https://github.com/redis-rs/redis-rs) for any struct or enum.

This allows seamless type conversion between Rust structs and Redis hash sets, which is more beneficial than JSON encoding the struct and storing the result in a Redis key because when saving as a Redis hash set, sorting algorithms can be performed without having to move data out of the database.

There is also the benefit of being able to retrieve just one value of the struct in the database.

Initial development was done by @Michaelvanstraten 🙏🏽.

### Features

- **RESP3 Support**: Native support for Redis 7+ protocol features including VerbatimString
- **Hash Field Expiration**: Per-field TTL support using Redis 7.4+ HEXPIRE commands
- **Client-Side Caching**: Automatic cache management with Redis 6+ client caching
- **Cluster Awareness**: Hash tag generation for Redis Cluster deployments
- **Flexible Naming**: Support for various case conversion rules (snake_case, kebab-case, etc.)
- **Comprehensive Error Handling**: Clear error messages for debugging
- **Performance Optimized**: Efficient serialization with minimal allocations

### Usage and Examples

Add this to your `Cargo.toml`:

```toml
[dependencies]
redis-derive = "0.2.0"
redis = "0.32"
```

Import the procedural macros:

```rust
use redis_derive::{FromRedisValue, ToRedisArgs};
```

#### Basic Struct Example

```rust
use redis::Commands;
use redis_derive::{FromRedisValue, ToRedisArgs};

#[derive(ToRedisArgs, FromRedisValue, Debug)]
struct User {
    id: u64,
    username: String,
    email: Option<String>,
    active: bool,
}

fn main() -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    let user = User {
        id: 12345,
        username: "john_doe".to_string(),
        email: Some("john@example.com".to_string()),
        active: true,
    };

    // Store individual fields
    con.hset("user:12345", "id", user.id)?;
    con.hset("user:12345", "username", &user.username)?;
    con.hset("user:12345", "email", &user.email)?;
    con.hset("user:12345", "active", user.active)?;

    // Retrieve the complete struct
    let retrieved_user: User = con.hgetall("user:12345")?;
    println!("Retrieved: {:?}", retrieved_user);

    Ok(())
}
```

#### Enum with Case Conversion

```rust
#[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq)]
#[redis(rename_all = "snake_case")]
enum UserRole {
    Administrator,      // stored as "administrator"
    PowerUser,          // stored as "power_user"
    RegularUser,        // stored as "regular_user"
    GuestUser,          // stored as "guest_user"
}

// Works seamlessly with Redis
let role = UserRole::PowerUser;
con.set("user:role", &role)?;
let retrieved: UserRole = con.get("user:role")?;
assert_eq!(role, retrieved);
```

### Naming Conventions and Attributes

#### Case Conversion Rules

The `rename_all` attribute supports multiple case conversion rules:

```rust
#[derive(ToRedisArgs, FromRedisValue)]
#[redis(rename_all = "snake_case")]
enum Status {
    InProgress,        // → "in_progress"
    WaitingForReview,  // → "waiting_for_review"
    Completed,         // → "completed"
}

#[derive(ToRedisArgs, FromRedisValue)]
#[redis(rename_all = "kebab-case")]
enum Priority {
    HighPriority,      // → "high-priority"
    MediumPriority,    // → "medium-priority"
    LowPriority,       // → "low-priority"
}
```

Supported case conversion rules:
- `"lowercase"`: `MyField` → `myfield`
- `"UPPERCASE"`: `MyField` → `MYFIELD`
- `"PascalCase"`: `my_field` → `MyField`
- `"camelCase"`: `my_field` → `myField`
- `"snake_case"`: `MyField` → `my_field`
- `"kebab-case"`: `MyField` → `my-field`

#### Important Naming Behavior

**Key insight**: The case conversion applies to **both** serialization and deserialization:

```rust
// With rename_all = "snake_case"
let role = UserRole::PowerUser;

// Serialization: PowerUser → "power_user"
con.set("key", &role)?;

// Deserialization: "power_user" → PowerUser
let retrieved: UserRole = con.get("key")?;

// Error messages also use converted names:
// "Unknown variant 'admin' for UserRole. Valid variants: [administrator, power_user, regular_user, guest_user]"
```

#### Redis Protocol Support

This crate handles multiple Redis value types automatically:

- **BulkString**: Most common for stored hash fields and string values
- **SimpleString**: Direct Redis command responses
- **VerbatimString**: Redis 6+ RESP3 protocol feature (automatically supported)
- **Proper error handling**: Clear messages for nil values and type mismatches

#### Advanced Features

##### Hash Field Expiration (Redis 7.4+)
```rust
#[derive(ToRedisArgs, FromRedisValue)]
struct SessionData {
    user_id: u64,
    #[redis(expire = "1800")] // 30 minutes
    access_token: String,
    #[redis(expire = "7200")] // 2 hours
    refresh_token: String,
}
```

##### Cluster-Aware Keys
```rust
#[derive(ToRedisArgs, FromRedisValue)]
#[redis(cluster_key = "user_id")]
struct UserProfile {
    user_id: u64,
    profile_data: String,
}
```

##### Client-Side Caching
```rust
#[derive(ToRedisArgs, FromRedisValue)]
#[redis(cache = true, ttl = "600")]
struct CachedData {
    id: u64,
    data: String,
}
```

### Development and Testing

The crate includes comprehensive examples in the `examples/` directory:

```bash
# Start Redis with Docker
cd examples && docker-compose up -d

# Run basic example
cargo run --example main

# Test all enum deserialization branches
cargo run --example enum_branches

# Debug attribute parsing behavior
cargo run --example debug_attributes
```

### Limitations

- Only unit enums (variants without fields) are currently supported
- Requires redis-rs 0.32.4 or later for full compatibility

### Compatibility

- **Redis**: Compatible with Redis 6+ (RESP2) and Redis 7+ (RESP3)
- **Rust**: MSRV 1.70+ (follows redis-rs requirements)
- **redis-rs**: 0.32.4+ (uses `num_of_args()` instead of deprecated `num_args()`)

License: MIT OR Apache-2.0

License: MIT OR Apache-2.0

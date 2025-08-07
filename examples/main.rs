use redis::Commands;
use redis_derive::{FromRedisValue, ToRedisArgs};
use std::collections::HashMap;

#[derive(FromRedisValue, ToRedisArgs, Debug, PartialEq)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(FromRedisValue, ToRedisArgs, Debug, PartialEq)]
#[redis(rename_all = "snake_case")]
enum UserRole {
    Administrator,
    Moderator,
    RegularUser,
}

#[derive(FromRedisValue, ToRedisArgs, Debug, PartialEq)]
struct User {
    id: u64,
    username: String,
    email: Option<String>,
    active: bool,
    favorite_color: Color,
    role: UserRole,
}

fn main() -> redis::RedisResult<()> {
    println!("🚀 Redis Derive Basic Example");
    println!("=============================");

    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_connection()?;

    // Test 1: Struct serialization/deserialization
    println!("\n📋 Testing struct serialization...");
    let user = User {
        id: 12345,
        username: "john_doe".to_string(),
        email: Some("john@example.com".to_string()),
        active: true,
        favorite_color: Color::Blue,
        role: UserRole::Administrator,
    };

    // Store the struct as a Redis hash using individual field sets
    con.hset::<_, _, _, ()>("user:12345", "id", user.id)?;
    con.hset::<_, _, _, ()>("user:12345", "username", &user.username)?;
    con.hset::<_, _, _, ()>("user:12345", "email", &user.email)?;
    con.hset::<_, _, _, ()>("user:12345", "active", user.active)?;
    con.hset::<_, _, _, ()>("user:12345", "favorite_color", &user.favorite_color)?;
    con.hset::<_, _, _, ()>("user:12345", "role", &user.role)?;


    // Retrieve the struct from Redis
    let retrieved_user: User = con.hgetall("user:12345")?;
    
    println!("   ✓ Original:  {:?}", user);
    println!("   ✓ Retrieved: {:?}", retrieved_user);
    assert_eq!(user, retrieved_user);
    println!("   ✅ Struct serialization works!");

    // Test 2: Enum serialization/deserialization
    println!("\n🎨 Testing enum serialization...");
    
    // Test Color enum (no rename_all)
    let _: () = con.set("user:color", &Color::Red)?;
    let color: Color = con.get("user:color")?;
    println!("   ✓ Color enum: {:?}", color);
    assert_eq!(color, Color::Red);

    // Test UserRole enum (with snake_case rename)
    let _: () = con.set("user:role", &UserRole::RegularUser)?;
    let role: UserRole = con.get("user:role")?;
    println!("   ✓ UserRole enum: {:?}", role);
    assert_eq!(role, UserRole::RegularUser);

    // Verify the actual stored value is snake_case
    let stored_role: String = con.get("user:role")?;
    println!("   ✓ Stored as: '{}'", stored_role);
    assert_eq!(stored_role, "regular_user"); // Now correctly snake_case!
    
    println!("   ✅ Enum serialization works!");

    // Test 3: View raw data
    println!("\n🔍 Inspecting stored data...");
    let hash_data: HashMap<String, String> = con.hgetall("user:12345")?;
    println!("   Raw hash data:");
    for (key, value) in &hash_data {
        println!("     {} = {}", key, value);
    }
    
    // Verify the role field in the hash is also snake_case
    assert_eq!(hash_data.get("role"), Some(&"administrator".to_string()));
    
    println!("   ✅ Hash field values are correctly transformed!");

    // Test 4: Individual field access
    println!("\n📎 Testing individual field access...");
    let username: String = con.hget("user:12345", "username")?;
    let role_str: String = con.hget("user:12345", "role")?;
    println!("   ✓ Username: {}", username);
    println!("   ✓ Role: {}", role_str);
    assert_eq!(username, "john_doe");
    assert_eq!(role_str, "administrator"); // snake_case conversion

    println!("\n🎉 All tests passed! Redis Derive is working correctly.");
    Ok(())
}
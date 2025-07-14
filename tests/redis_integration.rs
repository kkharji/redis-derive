// These tests require a running Redis server at localhost:6379
// Run with: cargo test --test redis_integration -- --ignored
// Or set REDIS_URL environment variable to a different Redis server

use redis::{Commands, Connection, RedisResult};
use redis_derive::{FromRedisValue, ToRedisArgs};
use std::collections::HashMap;

fn get_redis_connection() -> RedisResult<Connection> {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let client = redis::Client::open(redis_url)?;
    client.get_connection()
}

#[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq, Clone)]
struct User {
    id: u64,
    username: String,
    email: Option<String>,
    active: bool,
    score: f64,
    tags: Vec<String>,
}

#[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq, Clone)]
#[redis(rename_all = "snake_case")]
enum UserRole {
    Administrator,
    Moderator,
    RegularUser,
    GuestUser,
}

#[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq, Clone)]
struct UserProfile {
    user: User,
    role: UserRole,
    preferences: UserPreferences,
}

#[derive(ToRedisArgs, FromRedisValue, Debug, PartialEq, Clone)]
struct UserPreferences {
    theme: String,
    language: String,
    notifications_enabled: bool,
}

#[test]
#[ignore] // Requires Redis server
fn test_user_round_trip() -> RedisResult<()> {
    let mut con = get_redis_connection()?;
    
    let user = User {
        id: 12345,
        username: "testuser".to_string(),
        email: Some("test@example.com".to_string()),
        active: true,
        score: 95.5,
        tags: vec!["vip".to_string(), "premium".to_string()],
    };

    // Store user in Redis as a hash
    let key = "user:12345";
    let _: () = redis::cmd("HSET")
        .arg(key)
        .arg(&user)
        .query(&mut con)?;

    // Retrieve user from Redis
    let retrieved_user: User = con.hgetall(key)?;

    // Verify round trip works
    assert_eq!(user, retrieved_user);

    // Clean up
    let _: () = con.del(key)?;
    
    Ok(())
}

#[test]
#[ignore] // Requires Redis server
fn test_enum_round_trip() -> RedisResult<()> {
    let mut con = get_redis_connection()?;
    
    let role = UserRole::Administrator;
    let key = "role:admin";

    // Store enum as a simple string value
    let _: () = con.set(key, &role)?;

    // Retrieve enum from Redis
    let retrieved_role: UserRole = con.get(key)?;

    // Verify transformation works (should be "administrator" in Redis due to snake_case)
    assert_eq!(role, retrieved_role);

    // Also verify the actual stored value
    let stored_value: String = con.get(key)?;
    assert_eq!(stored_value, "administrator");

    // Clean up
    let _: () = con.del(key)?;
    
    Ok(())
}

#[test]
#[ignore] // Requires Redis server
fn test_nested_struct_round_trip() -> RedisResult<()> {
    let mut con = get_redis_connection()?;
    
    let profile = UserProfile {
        user: User {
            id: 67890,
            username: "nesteduser".to_string(),
            email: None,
            active: true,
            score: 88.2,
            tags: vec!["new".to_string()],
        },
        role: UserRole::Moderator,
        preferences: UserPreferences {
            theme: "dark".to_string(),
            language: "en-US".to_string(),
            notifications_enabled: true,
        },
    };

    let key = "profile:67890";
    let _: () = redis::cmd("HSET")
        .arg(key)
        .arg(&profile)
        .query(&mut con)?;

    let retrieved_profile: UserProfile = con.hgetall(key)?;
    assert_eq!(profile, retrieved_profile);

    // Clean up
    let _: () = con.del(key)?;
    
    Ok(())
}

#[test]
#[ignore] // Requires Redis server
fn test_optional_fields_handling() -> RedisResult<()> {
    let mut con = get_redis_connection()?;
    
    let user_with_email = User {
        id: 1,
        username: "with_email".to_string(),
        email: Some("user@test.com".to_string()),
        active: true,
        score: 100.0,
        tags: vec![],
    };

    let user_without_email = User {
        id: 2,
        username: "without_email".to_string(),
        email: None,
        active: false,
        score: 0.0,
        tags: vec![],
    };

    // Test user with email
    let key1 = "user:with_email";
    let _: () = redis::cmd("HSET").arg(key1).arg(&user_with_email).query(&mut con)?;
    let retrieved1: User = con.hgetall(key1)?;
    assert_eq!(user_with_email, retrieved1);

    // Test user without email
    let key2 = "user:without_email";
    let _: () = redis::cmd("HSET").arg(key2).arg(&user_without_email).query(&mut con)?;
    let retrieved2: User = con.hgetall(key2)?;
    assert_eq!(user_without_email, retrieved2);

    // Verify what's actually stored in Redis
    let hash1: HashMap<String, String> = con.hgetall(key1)?;
    assert!(hash1.contains_key("email"));
    
    let _hash2: HashMap<String, String> = con.hgetall(key2)?;
    // The None email might be stored as empty string or not at all, depending on implementation
    // This test verifies that deserialization handles it correctly regardless

    // Clean up
    let _: () = con.del(&[key1, key2])?;
    
    Ok(())
}

#[test]
#[ignore] // Requires Redis server
fn test_empty_collections() -> RedisResult<()> {
    let mut con = get_redis_connection()?;
    
    let user = User {
        id: 999,
        username: "empty_tags".to_string(),
        email: None,
        active: true,
        score: 50.0,
        tags: vec![], // Empty vector
    };

    let key = "user:empty_tags";
    let _: () = redis::cmd("HSET").arg(key).arg(&user).query(&mut con)?;
    let retrieved: User = con.hgetall(key)?;
    
    assert_eq!(user, retrieved);
    assert!(retrieved.tags.is_empty());

    // Clean up
    let _: () = con.del(key)?;
    
    Ok(())
}

#[test]
#[ignore] // Requires Redis server
fn test_unicode_content() -> RedisResult<()> {
    let mut con = get_redis_connection()?;
    
    let user = User {
        id: 888,
        username: "🚀unicode_user🌟".to_string(),
        email: Some("测试@example.com".to_string()),
        active: true,
        score: 77.7,
        tags: vec!["🏷️tag".to_string(), "العربية".to_string()],
    };

    let key = "user:unicode";
    let _: () = redis::cmd("HSET").arg(key).arg(&user).query(&mut con)?;
    let retrieved: User = con.hgetall(key)?;
    
    assert_eq!(user, retrieved);

    // Clean up
    let _: () = con.del(key)?;
    
    Ok(())
}

#[test]
#[ignore] // Requires Redis server
fn test_multiple_enum_values() -> RedisResult<()> {
    let mut con = get_redis_connection()?;
    
    let roles = vec![
        UserRole::Administrator,
        UserRole::Moderator,
        UserRole::RegularUser,
        UserRole::GuestUser,
    ];

    let expected_values = vec![
        "administrator",
        "moderator",
        "regular_user", 
        "guest_user",
    ];

    for (i, (role, expected)) in roles.iter().zip(expected_values.iter()).enumerate() {
        let key = format!("role:{}", i);
        
        // Store enum
        let _: () = con.set(&key, role)?;
        
        // Verify stored value is correctly transformed
        let stored: String = con.get(&key)?;
        assert_eq!(&stored, expected);
        
        // Verify round trip
        let retrieved: UserRole = con.get(&key)?;
        assert_eq!(role, &retrieved);
        
        // Clean up
        let _: () = con.del(&key)?;
    }
    
    Ok(())
}

#[test]
#[ignore] // Requires Redis server
fn test_large_struct_performance() -> RedisResult<()> {
    let mut con = get_redis_connection()?;
    
    // Create a user with large data
    let large_tags: Vec<String> = (0..1000).map(|i| format!("tag_{}", i)).collect();
    let user = User {
        id: 777,
        username: "large_data_user".to_string(),
        email: Some("large@example.com".to_string()),
        active: true,
        score: 99.9,
        tags: large_tags.clone(),
    };

    let key = "user:large";
    
    // Time the operations
    let start = std::time::Instant::now();
    let _: () = redis::cmd("HSET").arg(key).arg(&user).query(&mut con)?;
    let store_duration = start.elapsed();
    
    let start = std::time::Instant::now();
    let retrieved: User = con.hgetall(key)?;
    let retrieve_duration = start.elapsed();
    
    assert_eq!(user, retrieved);
    assert_eq!(retrieved.tags.len(), 1000);
    
    // Print performance info (will only show with --nocapture)
    println!("Store duration: {:?}", store_duration);
    println!("Retrieve duration: {:?}", retrieve_duration);
    
    // Clean up
    let _: () = con.del(key)?;
    
    Ok(())
}

// Helper function to set up test data in Redis
#[allow(dead_code)]
fn setup_test_data(con: &mut Connection) -> RedisResult<()> {
    // This could be used to populate Redis with test data
    // for more complex integration scenarios
    
    let test_users = vec![
        User {
            id: 1,
            username: "alice".to_string(),
            email: Some("alice@test.com".to_string()),
            active: true,
            score: 95.0,
            tags: vec!["admin".to_string()],
        },
        User {
            id: 2,
            username: "bob".to_string(),
            email: Some("bob@test.com".to_string()),
            active: true,
            score: 87.5,
            tags: vec!["user".to_string(), "premium".to_string()],
        },
    ];

    for user in test_users {
        let key = format!("test_user:{}", user.id);
        let _: () = redis::cmd("HSET").arg(key).arg(&user).query(con)?;
    }

    Ok(())
}

// Helper function to clean up test data
#[allow(dead_code)]
fn cleanup_test_data(con: &mut Connection) -> RedisResult<()> {
    let keys: Vec<String> = con.keys("test_user:*")?;
    if !keys.is_empty() {
        let _: () = con.del(keys)?;
    }
    Ok(())
}
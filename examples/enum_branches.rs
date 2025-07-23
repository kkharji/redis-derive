use redis::Commands;
use redis_derive::{FromRedisValue, ToRedisArgs};

#[derive(FromRedisValue, ToRedisArgs, Debug, PartialEq)]
#[redis(rename_all = "snake_case")]
enum UserRole {
    Administrator,
    Moderator,
    RegularUser,
    GuestUser,
}

#[derive(FromRedisValue, ToRedisArgs, Debug, PartialEq)]
enum Status {
    Active,
    Inactive,
    Pending,
}

fn main() -> redis::RedisResult<()> {
    println!("🚀 Redis Derive Enum Branches Example");
    println!("=====================================");

    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_connection()?;

    // Test 1: Normal BulkString case (most common)
    println!("\n1️⃣  Testing BulkString deserialization (normal case)");
    test_bulk_string_case(&mut con)?;

    // Test 2: SimpleString case (manual construction)
    println!("\n2️⃣  Testing SimpleString deserialization");
    test_simple_string_case()?;

    // Test 3: VerbatimString case (RESP3 feature)
    println!("\n3️⃣  Testing VerbatimString deserialization");
    test_verbatim_string_case()?;

    // Test 4: Nil value error case
    println!("\n4️⃣  Testing Nil value error handling");
    test_nil_case()?;

    // Test 5: Invalid type error case
    println!("\n5️⃣  Testing invalid type error handling");
    test_invalid_type_case()?;

    // Test 6: Invalid UTF-8 error case
    println!("\n6️⃣  Testing invalid UTF-8 error handling");
    test_invalid_utf8_case()?;

    // Test 7: Unknown variant error case
    println!("\n7️⃣  Testing unknown variant error handling");
    test_unknown_variant_case()?;

    println!("\n✅ All enum deserialization branches tested successfully!");
    Ok(())
}

fn test_bulk_string_case(con: &mut redis::Connection) -> redis::RedisResult<()> {
    // Store an enum value normally - this will be retrieved as BulkString
    let role = UserRole::Administrator;
    let _: () = con.set("user_role", &role)?;

    // Retrieve and deserialize - this triggers the BulkString branch
    let retrieved_role: UserRole = con.get("user_role")?;
    
    println!("   ✓ Stored: {:?}", role);
    println!("   ✓ Retrieved: {:?}", retrieved_role);
    assert_eq!(role, retrieved_role);
    println!("   ✓ BulkString deserialization works correctly");

    Ok(())
}

fn test_simple_string_case() -> redis::RedisResult<()> {
    // Manually construct a SimpleString value
    // Use the snake_case version since UserRole has rename_all = "snake_case"
    let simple_string_value = redis::Value::SimpleString("moderator".to_string());
    
    // Deserialize from SimpleString
    let result: redis::RedisResult<UserRole> = redis::FromRedisValue::from_redis_value(&simple_string_value);
    
    match result {
        Ok(role) => {
            println!("   ✓ SimpleString 'moderator' deserialized to: {:?}", role);
            assert_eq!(role, UserRole::Moderator);
            println!("   ✓ SimpleString deserialization works correctly");
        }
        Err(e) => {
            println!("   ❌ SimpleString deserialization failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

fn test_verbatim_string_case() -> redis::RedisResult<()> {
    // Manually construct a VerbatimString value (RESP3 feature)
    let verbatim_value = redis::Value::VerbatimString {
        format: redis::VerbatimFormat::Text,
        text: "regular_user".to_string(),
    };
    
    // Deserialize from VerbatimString
    let result: redis::RedisResult<UserRole> = redis::FromRedisValue::from_redis_value(&verbatim_value);
    
    match result {
        Ok(role) => {
            println!("   ✓ VerbatimString 'regular_user' deserialized to: {:?}", role);
            assert_eq!(role, UserRole::RegularUser);
            println!("   ✓ VerbatimString deserialization works correctly");
        }
        Err(e) => {
            println!("   ❌ VerbatimString deserialization failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

fn test_nil_case() -> redis::RedisResult<()> {
    // Test deserializing from Nil value
    let nil_value = redis::Value::Nil;
    
    let result: redis::RedisResult<UserRole> = redis::FromRedisValue::from_redis_value(&nil_value);
    
    match result {
        Ok(_) => {
            println!("   ❌ Expected error but got success!");
            panic!("Nil deserialization should fail");
        }
        Err(e) => {
            println!("   ✓ Nil value correctly rejected with error: {}", e);
            println!("   ✓ Nil error handling works correctly");
        }
    }

    Ok(())
}

fn test_invalid_type_case() -> redis::RedisResult<()> {
    // Test deserializing from incompatible type (Integer)
    let int_value = redis::Value::Int(42);
    
    let result: redis::RedisResult<UserRole> = redis::FromRedisValue::from_redis_value(&int_value);
    
    match result {
        Ok(_) => {
            println!("   ❌ Expected error but got success!");
            panic!("Integer deserialization should fail");
        }
        Err(e) => {
            println!("   ✓ Integer value correctly rejected with error: {}", e);
            println!("   ✓ Invalid type error handling works correctly");
        }
    }

    // Test with Array type too
    let array_value = redis::Value::Array(vec![
        redis::Value::SimpleString("not".to_string()),
        redis::Value::SimpleString("an".to_string()),
        redis::Value::SimpleString("enum".to_string()),
    ]);
    
    let result: redis::RedisResult<Status> = redis::FromRedisValue::from_redis_value(&array_value);
    
    match result {
        Ok(_) => {
            println!("   ❌ Expected error but got success!");
            panic!("Array deserialization should fail");
        }
        Err(e) => {
            println!("   ✓ Array value correctly rejected with error: {}", e);
            println!("   ✓ Array type error handling works correctly");
        }
    }

    Ok(())
}

fn test_invalid_utf8_case() -> redis::RedisResult<()> {
    // Create invalid UTF-8 bytes
    let invalid_utf8_bytes = vec![0xFF, 0xFE, 0xFD]; // Invalid UTF-8 sequence
    let bulk_string_value = redis::Value::BulkString(invalid_utf8_bytes);
    
    let result: redis::RedisResult<UserRole> = redis::FromRedisValue::from_redis_value(&bulk_string_value);
    
    match result {
        Ok(_) => {
            println!("   ❌ Expected UTF-8 error but got success!");
            panic!("Invalid UTF-8 deserialization should fail");
        }
        Err(e) => {
            println!("   ✓ Invalid UTF-8 correctly rejected with error: {}", e);
            println!("   ✓ UTF-8 validation works correctly");
        }
    }

    Ok(())
}

fn test_unknown_variant_case() -> redis::RedisResult<()> {
    // Test with unknown variant name
    let unknown_variant = redis::Value::SimpleString("super_admin".to_string());
    
    let result: redis::RedisResult<UserRole> = redis::FromRedisValue::from_redis_value(&unknown_variant);
    
    match result {
        Ok(_) => {
            println!("   ❌ Expected unknown variant error but got success!");
            panic!("Unknown variant deserialization should fail");
        }
        Err(e) => {
            println!("   ✓ Unknown variant 'super_admin' correctly rejected");
            println!("   ✓ Error message: {}", e);
            
            // Check that error message contains valid variants
            let error_msg = e.to_string();
            assert!(error_msg.contains("administrator"));
            assert!(error_msg.contains("moderator"));
            assert!(error_msg.contains("regular_user"));
            assert!(error_msg.contains("guest_user"));
            
            println!("   ✓ Error message includes valid variants list");
            println!("   ✓ Unknown variant error handling works correctly");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_variant_names() {
        // Test that our rename_all = "snake_case" works correctly
        let admin = UserRole::Administrator;
        let args = redis::ToRedisArgs::to_redis_args(&admin);
        assert_eq!(args[0], b"administrator");

        let regular = UserRole::RegularUser;
        let args = redis::ToRedisArgs::to_redis_args(&regular);
        assert_eq!(args[0], b"regular_user");
    }

    #[test]
    fn test_status_enum_no_rename() {
        // Test enum without rename_all
        let active = Status::Active;
        let args = redis::ToRedisArgs::to_redis_args(&active);
        assert_eq!(args[0], b"Active");
    }
}
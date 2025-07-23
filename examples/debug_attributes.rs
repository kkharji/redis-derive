use redis::Commands;
use redis_derive::{FromRedisValue, ToRedisArgs};

// Test enum WITHOUT rename_all
#[derive(FromRedisValue, ToRedisArgs, Debug, PartialEq)]
enum StatusNormal {
    Active,
    Inactive,
    Pending,
}

// Test enum WITH rename_all
#[derive(FromRedisValue, ToRedisArgs, Debug, PartialEq)]
#[redis(rename_all = "snake_case")]
enum StatusSnakeCase {
    VeryActive,
    SomewhatInactive,
    StillPending,
}

fn main() -> redis::RedisResult<()> {
    println!("🔍 Debug: Testing Attribute Parsing");
    println!("===================================");

    let client = redis::Client::open("redis://127.0.0.1:6379/")?;
    let mut con = client.get_connection()?;

    // Test 1: Normal enum (no rename_all)
    println!("\n1️⃣ Testing enum WITHOUT rename_all:");
    let status1 = StatusNormal::Active;
    let _: () = con.set("status_normal", &status1)?;
    let stored1: String = con.get("status_normal")?;
    println!("   ✓ StatusNormal::Active stored as: '{}'", stored1);
    
    let status2 = StatusNormal::Inactive;
    let args = redis::ToRedisArgs::to_redis_args(&status2);
    let arg_str = String::from_utf8(args[0].clone()).unwrap();
    println!("   ✓ StatusNormal::Inactive ToRedisArgs: '{}'", arg_str);

    // Test 2: Snake case enum (with rename_all)
    println!("\n2️⃣ Testing enum WITH rename_all = \"snake_case\":");
    let status3 = StatusSnakeCase::VeryActive;
    let _: () = con.set("status_snake", &status3)?;
    let stored3: String = con.get("status_snake")?;
    println!("   ✓ StatusSnakeCase::VeryActive stored as: '{}'", stored3);
    println!("   🔍 Expected: 'very_active' (if rename_all works)");
    
    let status4 = StatusSnakeCase::SomewhatInactive;
    let args = redis::ToRedisArgs::to_redis_args(&status4);
    let arg_str = String::from_utf8(args[0].clone()).unwrap();
    println!("   ✓ StatusSnakeCase::SomewhatInactive ToRedisArgs: '{}'", arg_str);
    println!("   🔍 Expected: 'somewhat_inactive' (if rename_all works)");

    // Test 3: Check if we can deserialize the stored values
    println!("\n3️⃣ Testing deserialization:");
    
    // Try to deserialize the normal enum
    let retrieved1: StatusNormal = con.get("status_normal")?;
    println!("   ✓ Retrieved StatusNormal: {:?}", retrieved1);
    assert_eq!(retrieved1, StatusNormal::Active);
    
    // Try to deserialize the snake case enum
    let retrieved3: StatusSnakeCase = con.get("status_snake")?;
    println!("   ✓ Retrieved StatusSnakeCase: {:?}", retrieved3);
    assert_eq!(retrieved3, StatusSnakeCase::VeryActive);

    // Test 4: Manual deserialization tests
    println!("\n4️⃣ Testing manual Value deserialization:");
    
    // Test if the snake case variant accepts snake_case strings
    let test_value = redis::Value::SimpleString("very_active".to_string());
    let result: redis::RedisResult<StatusSnakeCase> = redis::FromRedisValue::from_redis_value(&test_value);
    
    match result {
        Ok(val) => {
            println!("   ✓ 'very_active' successfully deserialized to: {:?}", val);
            println!("   ✅ rename_all is working for deserialization!");
        }
        Err(e) => {
            println!("   ❌ 'very_active' failed to deserialize: {}", e);
            println!("   🔍 rename_all might not be working...");
            
            // Try with original case
            let test_value2 = redis::Value::SimpleString("VeryActive".to_string());
            let result2: redis::RedisResult<StatusSnakeCase> = redis::FromRedisValue::from_redis_value(&test_value2);
            
            match result2 {
                Ok(val) => {
                    println!("   ✓ 'VeryActive' (original case) deserialized to: {:?}", val);
                    println!("   ⚠️  rename_all is NOT working - using original enum names");
                }
                Err(e2) => {
                    println!("   ❌ Both 'very_active' and 'VeryActive' failed!");
                    println!("      Error 1: {}", e);
                    println!("      Error 2: {}", e2);
                }
            }
        }
    }

    println!("\n🔍 Debug complete!");
    Ok(())
}
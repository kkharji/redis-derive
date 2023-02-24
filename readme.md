# redis-derive

This crate implements the `FromRedisValue`(redis::FromRedisValue) and `ToRedisArgs`(redis::ToRedisArgs) traits from `mitsuhiko / redis-rs`(https://github.com/mitsuhiko/redis-rs) for any struct,
this allows a seaming less type conversion between rust structs and Redis hash sets.

This is more beneficial than JSON encoding the struct and storing the result in a Redis key because when saving as a Redis hash set,
sorting algorithms can be performed without having to move data out of the database.

There is also the benefit of being able to retrieve just one value of the struct in the database.

Initial development was done by @Michaelvanstraten 🙏🏽.

## Usage and Examples

To use this crate at it to your dependencies and import the following to procedural macros.

```rust
use redis_derive::{FromRedisValue, ToRedisArgs};
```

Now the these Marcos can be used to implement the traits `FromRedisValue`(redis::FromRedisValue) and `ToRedisArgs`(redis::ToRedisArgs) for your decorated struct.

```rust
#[derive(ToRedisArgs, FromRedisValue)]
struct MySuperCoolStruct {
    first_field : String,
    second_field : Option<i64>,
    third_field : Vec<String>
}

```
These Procedural macros work for any struct in which every field's type also implements `ToRedisArgs`(redis::ToRedisArgs) so this would be allowed:
```rust
#[derive(ToRedisArgs, FromRedisVaule)]
struct MySuperCoolStruct {
    first_field : String,
    second_field : Option<i64>,
    third_field : Vec<String>
}

#[derive(ToRedisArgs, FromRedisVaule)]
struct MySecondSuperCoolStruct {
    fourth_field : String,
    inner_struct : MySuperCoolStruct
}
```
### Complete Example
```rust
use redis::Commands;
use redis_derive::{FromRedisValue, ToRedisArgs};

#[derive(FromRedisValue, ToRedisArgs, Debug)]
struct MySuperCoolStruct {
    first_field : String,
    second_field : Option<i64>,
    third_field : Vec<String>
}

fn main() -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    let test1 = MySuperCoolStruct{
        first_field : "Hello World".to_owned(),
        second_field : Some(42),
        third_field : vec!["abc".to_owned(), "cba".to_owned()]
    };

    let _ = redis::cmd("HSET")
        .arg("test1")
        .arg(&test1)
        .query(&mut con)?;

    let db_test1 : MySuperCoolStruct = con.hgetall("test1")?;

    println!("send : {:#?}, got : {:#?}", test1, db_test1);
    Ok(())
}
```

License: MIT OR Apache-2.0

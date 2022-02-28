# redis-derive

This crate allows a seamingless type conversiom between rust structs and redis hashsets. This i more benifical than json encoding the struct and stroring the result in a redis key because when saving as a redis hashset sorting algorithems can be performt without having to move data out of the databaser. There is also the benifit of being able to retrieve just one value of the struct in the database.

## Usage 

[ mitsuhiko / redis-rs](https://github.com/mitsuhiko/redis-rs)

To use this crate at it to your dependencies and import the following to procidual macros.

```rust
    use redis_derive::{FromRedisValue, ToRedisArgs};
``` 

Now the these marcos can be used to implements the traits ```redis::FromRedisValue``` and ```redis::ToRedisArgs``` for your decorated struct.

```rust
#[derive(ToRedisArgs, FromRedisVaule)]
struct MySuperCoolStruct {
    first_field : String,
    second_field : Option<i64>,
    third_field : Vec<String>
}

```
These Procidual macros work for any struct in which every fields type also implements ToRedisArgs. So this would be allow: 
```rust 
#[derive(ToRedisArgs, FromRedisVaule)]
struct MySuperCoolStruct {
    first_field : String,
    second_field : Option<i64>,
    third_field : Vec<String>
}

#[derive(ToRedisArgs, FromRedisVaule)]
struct MySecondSuperCoolStruct {
    fourth_field : Strin,
    inner_struct : MySuperCoolStruct
}
```

## Problems and future continuasions 

This Macro at this point sadly dosen't support enums. 

### Future Continuation

- implementing a getter and setter for a redis derived type, i imagen something like this 
```rust
    #[derive(RedisGetter, RedisSetter)]
    struct MySuperCoolStruct {
        first_field : String,
        second_field : Option<i64>,
        third_field : Vec<String>
    }
    fn somefn() {
        let mut redis_client = /* geting some connection to redis db */;
        let first_field : String = MySuperCoolStruct::first_field::get(&redis_client, key : "MyRedisKeyForStruct");
        MySuperCoolStruct::first_field::set(&redis_client, key : "MyRedisKeyForStruct", value : String::from("test"));
    }
```

 
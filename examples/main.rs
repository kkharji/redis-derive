use redis_derive::{FromRedisValue, ToRedisArgs};

use std::collections::HashMap;

use redis::Commands;

#[derive(FromRedisValue, ToRedisArgs, Debug)]
enum Color {
    Red,
    Green,
}

#[derive(Default, FromRedisValue, ToRedisArgs, Debug)]
#[redis(rename_all = "snake_case")]
enum Group {
    #[default]
    MemberGroup,
    AdminGroup,
}

#[derive(FromRedisValue, ToRedisArgs, Debug)]
#[redis(rename_all = "camelCase")]
struct MySuperCoolStruct {
    #[redis(rename = "id")]
    first_field: String,
    second_field: Option<i64>,
    third_field: Vec<String>,
    color: Color,
    group: Group,
}

fn main() -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    let test1 = MySuperCoolStruct {
        first_field: "Hello World".to_owned(),
        second_field: Some(42),
        third_field: vec!["abc".to_owned(), "cba".to_owned()],
        color: Color::Red,
        group: Group::AdminGroup,
    };

    let _ = redis::cmd("HSET")
        .arg("test1")
        .arg(&test1)
        .query(&mut con)?;

    let db_test1: MySuperCoolStruct = con.hgetall("test1")?;

    println!("send : {:#?}, got : {:#?}", test1, db_test1);

    let db_test1: HashMap<String, String> = con.hgetall("test1")?;
    assert_eq!(db_test1["group"], "admin_group");
    assert_eq!(db_test1["id"], "Hello World");
    assert_eq!(db_test1["secondField"], "42");

    Ok(())
}

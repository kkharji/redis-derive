/*!
This crate implements the [FromRedisValue](redis::FromRedisValue) and [ToRedisArgs](redis::ToRedisArgs) traits from
  for any struct,
 this allows a seaming less type conversion between rust structs and Redis hash sets.

 This is more beneficial than JSON encoding the struct and storing the result in a Redis key because when saving as a Redis hash set,
 sorting algorithms can be performed without having to move data out of the database.

 There is also the benefit of being able to retrieve just one value of the struct in the database.

 ## Example
 ```
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
*/
use proc_macro::TokenStream;
use syn::{
    parse_macro_input,
    Data::{Enum, Struct, Union},
    DeriveInput, Ident,
};
use util::ParsedAttributeMap;

mod data_enum;
mod data_struct;
mod util;

#[proc_macro_derive(ToRedisArgs, attributes(redis))]
/// This Derive Macro is responsible for Implementing the [ToRedisArgs](redis::ToRedisArgs) trait for the decorated struct.
pub fn to_redis_args(tokenstream: TokenStream) -> TokenStream {
    let abstract_syntax_tree = parse_macro_input!(tokenstream as DeriveInput);
    let type_identifier = abstract_syntax_tree.ident;
    let attr_map = util::parse_attributes(&abstract_syntax_tree.attrs);

    match abstract_syntax_tree.data {
        Struct(data_struct) => data_struct.derive_to_redis(type_identifier, attr_map),
        Enum(data_enum) => data_enum.derive_to_redis(type_identifier, attr_map),
        Union(_) => todo!(),
    }
}

#[proc_macro_derive(FromRedisValue, attributes(redis))]
/// This Derive Macro is responsible for Implementing the [ToRedisArgs](redis::FromRedisValue) trait for the decorated struct.
pub fn from_redis_value(tokenstream: TokenStream) -> TokenStream {
    let abstract_syntax_tree = parse_macro_input!(tokenstream as DeriveInput);
    let type_identifier = abstract_syntax_tree.ident;
    let attr_map = util::parse_attributes(&abstract_syntax_tree.attrs);

    match abstract_syntax_tree.data {
        Struct(data_struct) => data_struct.derive_from_redis(type_identifier, attr_map),
        Enum(data_enum) => data_enum.derive_from_redis(type_identifier, attr_map),
        Union(_) => todo!(),
    }
}

trait DeriveToRedisArgs {
    fn derive_to_redis(&self, type_ident: Ident, attrs: ParsedAttributeMap) -> TokenStream;
}

trait DeriveFromRedisArgs {
    fn derive_from_redis(&self, type_ident: Ident, attrs: ParsedAttributeMap) -> TokenStream;
}

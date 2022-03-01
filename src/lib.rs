//! # redis-derive

//! This crate implements the ```redis::FromRedisValue``` and ```redis::ToRedisArgs``` traits from [mitsuhiko / redis-rs](https://github.com/mitsuhiko/redis-rs) for any struct, 
//! this allows a seaming less type conversion between rust structs and Redis hash sets. 
//! 
//! This is more beneficial than JSON encoding the struct and storing the result in a Redis key because when saving as a Redis hash set, 
//! sorting algorithms can be performed without having to move data out of the database. 
//! 
//! There is also the benefit of being able to retrieve just one value of the struct in the database.
//! 
//! ## Example
//! ```
//! use redis::Commands;
//! use redis_derive::{FromRedisValue, ToRedisArgs};
//! 
//! #[derive(FromRedisValue, ToRedisArgs, Debug)]
//! struct MySuperCoolStruct {
//!     first_field : String,
//!     second_field : Option<i64>,
//!     third_field : Vec<String>
//! }
//! 
//! fn main() -> redis::RedisResult<()> {
//!     let client = redis::Client::open("redis://127.0.0.1/")?;
//!     let mut con = client.get_connection()?;
//! 
//!     let test1 = MySuperCoolStruct{
//!         first_field : "Hello World".to_owned(),
//!         second_field : Some(42),
//!         third_field : vec!["abc".to_owned(), "cba".to_owned()]
//!     };
//! 
//!     let _ = redis::cmd("HMSET")
//!         .arg("test1")
//!         .arg(&test1)
//!         .query(&mut con)?;
//! 
//!     let db_test1 : MySuperCoolStruct = con.hgetall("test1")?;
//! 
//!     println!("send : {:#?}, got : {:#?}", test1, db_test1);
//!     Ok(())
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, 
    DeriveInput, Data::Struct, 
    DataStruct, 
    Fields, 
    FieldsNamed
};



#[proc_macro_derive(ToRedisArgs)]
/// This Derive Macro is responsible for Implementing the [`redis::ToRedisArgs`] trait for the decorated struct.
pub fn to_redis_args(tokenstream : TokenStream) -> TokenStream {
    let abstract_syntax_tree = parse_macro_input!(tokenstream as DeriveInput);
    let struct_idententifier = abstract_syntax_tree.ident;
    let (field_idententifier_strs, field_idententifiers) = derive_fields(abstract_syntax_tree.data);
    quote!{
        impl redis::ToRedisArgs for #struct_idententifier {
            fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                let mut redis_args : Vec<Vec<u8>> = Vec::new();
                #(
                    redis_args = self.#field_idententifiers.to_redis_args();
                    match redis_args.len() {
                        0 => (),
                        1 => {
                            out.write_arg_fmt(#field_idententifier_strs);
                            out.write_arg(&redis_args[0]);
                        },
                        n => {
                            for args in redis_args.chunks(2) {
                                out.write_arg_fmt(format!("{}.{}", #field_idententifier_strs, String::from_utf8(args[0].clone()).unwrap()));
                                out.write_arg(&args[1])
                            }
                        }
                    }
                )*
            }
        }
    }.into()
}


#[proc_macro_derive(FromRedisValue)]
/// This Derive Macro is responsible for Implementing the [`redis::FromRedisValue`] trait for the decorated struct.
pub fn from_redis_value(tokenstream : TokenStream) -> TokenStream {
    let abstract_syntax_tree = parse_macro_input!(tokenstream as DeriveInput);
    let struct_idententifier = abstract_syntax_tree.ident;
    let (field_idententifier_strs, field_identifiers) = derive_fields(abstract_syntax_tree.data);

    quote!{
        impl redis::FromRedisValue for #struct_idententifier {
            fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                match v {
                    redis::Value::Bulk(bulk_data) if bulk_data.len() % 2 == 0 => {
                        let mut fields_hashmap : std::collections::HashMap<String, redis::Value> = std::collections::HashMap::new();
                        for values in bulk_data.chunks(2) {
                            let full_identifier : String = redis::from_redis_value(&values[0])?;
                            match full_identifier.split_once('.') {
                                Some((field_identifier, split_of_section)) => {
                                    match fields_hashmap.get_mut(field_identifier) {
                                        Some(redis::Value::Bulk(bulk)) => {
                                            bulk.push(redis::Value::Data(split_of_section.chars().map(|c| c as u8).collect()));
                                            bulk.push(values[1].clone())
                                        },
                                        _ => {
                                            let mut new_bulk : Vec<redis::Value> = Vec::new();
                                            new_bulk.push(redis::Value::Data(split_of_section.chars().map(|c| c as u8).collect()));
                                            new_bulk.push(values[1].clone());
                                            fields_hashmap.insert(field_identifier.to_owned(), redis::Value::Bulk(new_bulk));
                                        }
                                    }
                                },
                                None => {
                                    fields_hashmap.insert(full_identifier, values[1].clone());
                                }
                            }
                        }   
                        Ok(
                            Self {
                                #(
                                    #field_identifiers : redis::from_redis_value(
                                        fields_hashmap.get(
                                            #field_idententifier_strs
                                        )
                                        .unwrap_or(&redis::Value::Nil)
                                    )?,
                                )*
                            }
                        )
                    },
                    _ => Err(
                        redis::RedisError::from((
                            redis::ErrorKind::TypeError, 
                            "the data returned from the redis database was not in the bulk data format or the length of the bulk data is not devisable by two"))
                    )
                }
            }
        }
    }.into()
}


/// This function is used to extract the Identifier and and Stringified version of the Ident and Map it to a Tuple of To Vectors,
/// this is used later to populate the fields in the struct as well as set and query the values from redis.
/// ```
/// if let Struct(
///     DataStruct{
///         fields : Fields::Named(
///             FieldsNamed{
///                 named,
///                 ..},
///             ..),
///         ..
///     }
/// ) = data {
///     named
///         .into_iter()
///         .map(
///             |field| {
///                 let field_idententifier = field.ident.unwrap();
///                 (
///                     format!("{}", field_idententifier),
///                     field_idententifier
///                 )
///             }
///         )
///         .unzip()
/// } else {
///     unimplemented!()
/// }
/// ```

fn derive_fields(data : syn::Data) -> (Vec<String>, Vec<syn::Ident>) {
    if let Struct(
        DataStruct{
            fields : Fields::Named(
                FieldsNamed{
                    named,
                    ..},
                ..),
            ..
        }
    ) = data {
        named
            .into_iter()
            .map(
                |field| {
                    let field_idententifier = field.ident.unwrap();
                    (
                        format!("{}", field_idententifier),
                        field_idententifier
                    )
                }
            )
            .unzip()
    } else {
        unimplemented!()
    }
}

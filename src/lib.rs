
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data::Struct, DataStruct, Fields, FieldsNamed};

#[proc_macro_derive(ToRedisArgs)]
pub fn to_redis_args(input : TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let idententifier = &ast.ident;
    let mut first_item : bool = true;
    let fields = if let Struct(DataStruct{fields : Fields::Named(FieldsNamed{named,..},..),..}) = ast.data {
        named.into_iter().map(|field|{
            let field_ident = field.ident.as_ref().unwrap();
            let str_field_ident = format!("{}", field_ident);
            if first_item {
                first_item = false;
                quote!{
                    let mut redis_args = self.#field_ident.to_redis_args();
                    if redis_args.len() > 0 {
                        out.write_arg_fmt(#str_field_ident);
                        for arg in redis_args {
                            out.write_arg(&arg)
                        }                                
                    };
                }
            } else {
                quote!{
                    redis_args = self.#field_ident.to_redis_args();
                    if redis_args.len() > 0 {
                        out.write_arg_fmt(#str_field_ident);
                        for arg in redis_args {
                            out.write_arg(&arg)
                        }                                
                    };
                }
            }
        })
    } else {
        unimplemented!()
    };
    quote!{
        impl redis::ToRedisArgs for #idententifier {
            fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                #(#fields)*
            }
        }
    }.into()
}

#[proc_macro_derive(FromRedisValue)]
pub fn from_redis_value(input : TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_idententifier = &ast.ident;
    let fields = if let Struct(DataStruct { fields : Fields::Named(FieldsNamed{named, .. } , ..), ..}) = ast.data {
        named.into_iter().map(|field| {
            let field_ident = field.ident.as_ref().unwrap();
            let str_field_ident = format!("{}", field_ident);
            quote!{
                #field_ident : redis::from_redis_value(field_hashmap.get(#str_field_ident).unwrap_or(&&redis::Value::Nil))?
            }
        })
    } else {
        unimplemented!()
    };
    quote!{
        impl redis::FromRedisValue for #struct_idententifier {
            fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                match v {
                    redis::Value::Bulk(bulk) => {
                        if bulk.len() % 2 == 0 {
                            let field_hashmap : std::collections::HashMap<String, &redis::Value> = bulk.chunks(2).into_iter().map(|(values)| {
                                (redis::from_redis_value(&values[0]).unwrap(), &values[1])
                            }).collect();
                            Ok(
                                Self {
                                    #(#fields,)*
                                }
                            )
                        } else {
                            Err(redis::RedisError::from((redis::ErrorKind::TypeError, "Data from, return from redis database was corupted")))
                        }
                    },
                    _ => Err(redis::RedisError::from((redis::ErrorKind::TypeError, "asdasd")))
                }
            }
        }
    }.into()
}
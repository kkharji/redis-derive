use super::{DeriveFromRedisArgs, DeriveToRedisArgs};
use crate::util::ParsedAttributeMap;
use quote::quote;
use syn::{DataStruct, Fields, Ident};

impl DeriveToRedisArgs for DataStruct {
    fn derive_to_redis(
        &self,
        type_ident: Ident,
        _attrs: ParsedAttributeMap,
    ) -> proc_macro::TokenStream {
        match &self.fields {
            Fields::Named(fields_named) => {
                let (stringified_idents, idents): (Vec<String>, Vec<&Ident>) = fields_named
                    .named
                    .iter()
                    .map(|named_field| {
                        let ident = named_field
                            .ident
                            .as_ref()
                            .expect("there should be an ident on this field");
                        (ident.to_string(), ident)
                    })
                    .unzip();

                quote! {
                    impl redis::ToRedisArgs for #type_ident {
                        fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                            #(
                                match self.#idents.to_redis_args() {
                                    redis_args if redis_args.len() == 1 => {
                                        out.write_arg_fmt(#stringified_idents);
                                        out.write_arg(&redis_args[0]);
                                    },
                                    redis_args => {
                                        for args in redis_args.chunks(2) {
                                            out.write_arg_fmt(format!("{}.{}", #stringified_idents, String::from_utf8(args[0].clone()).unwrap()));
                                            out.write_arg(&args[1])
                                        }
                                    }
                                }
                            )*
                        }
                    }
                }.into()
            }
            Fields::Unnamed(fields_unnamed) => {
                let (stringified_idents, idents): (Vec<String>, Vec<usize>) =
                    (0..fields_unnamed.unnamed.len())
                        .into_iter()
                        .map(|index| (index.to_string(), index))
                        .unzip();
                quote!{
                    impl redis::ToRedisArgs for #type_ident {
                        fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                            #(
                                match self.#idents.to_redis_args() {
                                    redis_args if redis_args.len() == 1 => {
                                        out.write_arg_fmt(#stringified_idents);
                                        out.write_arg(&redis_args[0]);
                                    },
                                    redis_args => {
                                        for args in redis_args.chunks(2) {
                                            out.write_arg_fmt(format!("{}.{}", #stringified_idents, String::from_utf8(args[0].clone()).unwrap()));
                                            out.write_arg(&args[1])
                                        }
                                    }
                                }
                            )*
                        }
                    }
                }.into()
            }
            Fields::Unit => quote! {
                impl redis::ToRedisArgs for #type_ident {
                    fn write_redis_args<W : ?Sized + redis::RedisWrite>(&self, out: &mut W) {}
                }
            }
            .into(),
        }
    }
}

impl DeriveFromRedisArgs for DataStruct {
    fn derive_from_redis(
        &self,
        type_ident: Ident,
        _attrs: ParsedAttributeMap,
    ) -> proc_macro::TokenStream {
        match &self.fields {
            Fields::Named(fields_named) => {
                let (stringified_idents, idents): (Vec<String>, Vec<&Ident>) = fields_named
                    .named
                    .iter()
                    .map(|named_field| {
                        let ident = named_field
                            .ident
                            .as_ref()
                            .expect("there should be an ident on this field");
                        (ident.to_string(), ident)
                    })
                    .unzip();

                quote! {
                    impl redis::FromRedisValue for #type_ident {
                        fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                            match v {
                                redis::Value::Bulk(bulk_data) if bulk_data.len() % 2 == 0 => {
                                    let mut fields_hashmap = std::collections::HashMap::new();
                                    for values in bulk_data.chunks(2) {
                                        let full_identifier : String = redis::from_redis_value(&values[0])?;
                                        match full_identifier.split_once(".") {
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
                                    Ok(Self {
                                        #(#idents: redis::from_redis_value(
                                                fields_hashmap.get(
                                                    #stringified_idents
                                                )
                                                .unwrap_or(&redis::Value::Nil)
                                            )?,
                                        )*
                                    })
                                },
                                _ => Err(redis::RedisError::from((
                                    redis::ErrorKind::TypeError,
                                    "the data returned from the redis database was not in the bulk data format or the length of the bulk data is not devisable by two"))
                                )
                            }
                        }
                    }
                }.into()
            }
            Fields::Unnamed(fields_unnamed) => {
                let (stringified_idents, idents): (Vec<String>, Vec<usize>) =
                    (0..fields_unnamed.unnamed.len())
                        .into_iter()
                        .map(|index| (index.to_string(), index))
                        .unzip();
                quote! {
                    impl redis::FromRedisValue for #type_ident {
                        fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                            match v {
                                redis::Value::Bulk(bulk_data) if bulk_data.len() % 2 == 0 => {
                                    let mut fields_hashmap = std::collections::HashMap::new();
                                    for values in bulk_data.chunks(2) {
                                        let full_identifier : String = redis::from_redis_value(&values[0])?;
                                        match full_identifier.split_once(".") {
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
                                    Ok(Self {
                                        #(
                                            #idents: redis::from_redis_value(fields_hashmap.get(#stringified_idents)
                                                .unwrap_or(&redis::Value::Nil)
                                                )?,
                                        )*
                                    })
                                },
                                _ => Err(redis::RedisError::from((
                                    redis::ErrorKind::TypeError,
                                    "the data returned from the redis database was not in the bulk data format or the length of the bulk data is not devisable by two"))
                                )
                            }
                        }
                    }
                }.into()
            }
            Fields::Unit => quote! {
                impl redis::FromRedisValue for #type_ident {
                    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                        #type_ident{}
                    }
                }
            }
            .into(),
        }
    }
}

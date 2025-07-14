use crate::util::ParsedAttributeMap;
use crate::{DeriveFromRedisArgs, DeriveToRedisArgs};

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
                        fn write_redis_args<W: ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                            #(
                                out.write_arg(#stringified_idents.as_bytes());
                                (&self.#idents).write_redis_args(out);
                            )*
                        }
                    }
                }
                .into()
            }
            Fields::Unnamed(fields_unnamed) => {
                let indices: Vec<syn::Index> = (0..fields_unnamed.unnamed.len())
                    .map(syn::Index::from)
                    .collect();

                quote! {
                    impl redis::ToRedisArgs for #type_ident {
                        fn write_redis_args<W: ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                            #(
                                (&self.#indices).write_redis_args(out);
                            )*
                        }
                    }
                }
                .into()
            }
            Fields::Unit => quote! {
                impl redis::ToRedisArgs for #type_ident {
                    fn write_redis_args<W: ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                        out.write_arg(&[]);
                    }
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
                        fn from_redis_value(v: &redis::Value) -> Result<Self, redis::RedisError> {
                            use redis::{ ErrorKind, Value };
                            
                            let Value::Map(ref items) = *v else {
                                return Err((ErrorKind::TypeError, "Expected Map for struct").into());
                            };

                            let mut h = std::collections::HashMap::new();
                            for (k, v) in items {
                                let k: String = redis::FromRedisValue::from_redis_value(k)?;
                                h.insert(k, v);
                            }

                            Ok(#type_ident {
                                #(
                                    #idents: match h.get(#stringified_idents) {
                                        Some(v) => redis::FromRedisValue::from_redis_value(v)?,
                                        None => redis::FromRedisValue::from_redis_value(&redis::Value::Nil)?,
                                    },
                                )*
                            })
                        }
                    }
                }
                .into()
            }
            Fields::Unnamed(fields_unnamed) => {
                let field_count = fields_unnamed.unnamed.len();
                let indices: Vec<syn::Index> = (0..field_count)
                    .map(syn::Index::from)
                    .collect();

                quote! {
                    impl redis::FromRedisValue for #type_ident {
                        fn from_redis_value(v: &redis::Value) -> Result<Self, redis::RedisError> {
                            use redis::{ ErrorKind, Value };
                            
                            let Value::Array(ref items) = *v else {
                                return Err((ErrorKind::TypeError, "Expected Array for tuple struct").into());
                            };

                            if items.len() != #field_count {
                                return Err((ErrorKind::TypeError, "Wrong array length for tuple struct").into());
                            }

                            Ok(#type_ident(
                                #(
                                    redis::FromRedisValue::from_redis_value(&items[#indices])?,
                                )*
                            ))
                        }
                    }
                }
                .into()
            }
            Fields::Unit => {
                quote! {
                    impl redis::FromRedisValue for #type_ident {
                        fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                            Ok(#type_ident)
                        }
                    }
                }
                .into()
            }
        }
    }
}
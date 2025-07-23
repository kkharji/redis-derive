use crate::util::{self, ParsedAttributeMap};
use quote::quote;
use syn::{DataStruct, Fields, Ident};

pub fn derive_to_redis_struct(
    data_struct: DataStruct,
    type_ident: Ident,
    attrs: ParsedAttributeMap,
) -> proc_macro::TokenStream {
    match &data_struct.fields {
        Fields::Named(fields_named) => {
            let mut regular_fields = Vec::new();

            for field in &fields_named.named {
                let field_ident = field.ident.as_ref().expect("Named field should have ident");
                let field_attrs = util::parse_field_attributes(&field.attrs);

                if field_attrs.skip {
                    continue;
                }

                let field_name = util::transform_field_name(
                    &field_ident.to_string(),
                    attrs.rename_all.as_ref(),
                    field_attrs.rename.as_ref(),
                );

                regular_fields.push((field_ident, field_name.clone()));
            }

            let (field_idents, field_names): (Vec<_>, Vec<_>) =
                regular_fields.into_iter().unzip();

            // Generate the basic ToRedisArgs implementation
            let to_redis_impl = quote! {
                impl redis::ToRedisArgs for #type_ident {
                    fn write_redis_args<W: ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                        // Write each field as key-value pairs for hash storage
                        #(
                            out.write_arg(#field_names.as_bytes());
                            (&self.#field_idents).write_redis_args(out);
                        )*
                    }

                    fn num_of_args(&self) -> usize {
                        let mut count = 0;
                        #(
                            count += 1; // field name
                            count += (&self.#field_idents).num_of_args(); // field value args
                        )*
                        count
                    }
                }
            };

            to_redis_impl.into()
        }
        Fields::Unnamed(fields_unnamed) => {
            let field_count = fields_unnamed.unnamed.len();
            let indices: Vec<usize> = (0..field_count).collect();

            let to_redis_impl = quote! {
                impl redis::ToRedisArgs for #type_ident {
                    fn write_redis_args<W: ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                        // Write tuple struct fields as an array
                        #(
                            (&self.#indices).write_redis_args(out);
                        )*
                    }

                    fn num_of_args(&self) -> usize {
                        let mut count = 0;
                        #(
                            count += (&self.#indices).num_of_args();
                        )*
                        count
                    }
                }
            };

            to_redis_impl.into()
        }
        Fields::Unit => {
            let to_redis_impl = quote! {
                impl redis::ToRedisArgs for #type_ident {
                    fn write_redis_args<W: ?Sized + redis::RedisWrite>(&self, _out: &mut W) {
                        // Unit structs don't write any args
                    }

                    fn num_of_args(&self) -> usize {
                        0
                    }
                }
            };

            to_redis_impl.into()
        }
    }
}

pub fn derive_from_redis_struct(
    data_struct: DataStruct,
    type_ident: Ident,
    attrs: ParsedAttributeMap,
) -> proc_macro::TokenStream {
    match &data_struct.fields {
        Fields::Named(fields_named) => {
            let mut regular_fields = Vec::new();

            for field in &fields_named.named {
                let field_ident = field.ident.as_ref().expect("Named field should have ident");
                let field_attrs = util::parse_field_attributes(&field.attrs);

                if field_attrs.skip {
                    continue;
                }

                let field_name = util::transform_field_name(
                    &field_ident.to_string(),
                    attrs.rename_all.as_ref(),
                    field_attrs.rename.as_ref(),
                );

                regular_fields.push((field_ident, field_name));
            }

            let (field_idents, field_names): (Vec<_>, Vec<_>) =
                regular_fields.into_iter().unzip();

            let from_redis_impl = quote! {
                impl redis::FromRedisValue for #type_ident {
                    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                        match v {
                            redis::Value::Array(items) if items.len() % 2 == 0 => {
                                let mut fields_map = std::collections::HashMap::new();
                                
                                // Parse key-value pairs from array
                                for chunk in items.chunks(2) {
                                    let key: String = redis::FromRedisValue::from_redis_value(&chunk[0])?;
                                    fields_map.insert(key, &chunk[1]);
                                }

                                Ok(Self {
                                    #(
                                        #field_idents: {
                                            match fields_map.get(#field_names) {
                                                Some(value) => redis::FromRedisValue::from_redis_value(value)
                                                    .map_err(|e| redis::RedisError::from((
                                                        redis::ErrorKind::TypeError,
                                                        "Failed to parse field",
                                                        format!("Field '{}': {}", #field_names, e),
                                                    )))?,
                                                None => return Err(redis::RedisError::from((
                                                    redis::ErrorKind::TypeError,
                                                    "Missing required field",
                                                    #field_names.to_string(),
                                                ))),
                                            }
                                        },
                                    )*
                                })
                            }
                            redis::Value::Map(map) => {
                                // Handle Redis hash/map type (RESP3)
                                let mut fields_map = std::collections::HashMap::new();
                                
                                for (key, value) in map {
                                    let key_str: String = redis::FromRedisValue::from_redis_value(key)?;
                                    fields_map.insert(key_str, value);
                                }

                                Ok(Self {
                                    #(
                                        #field_idents: {
                                            match fields_map.get(#field_names) {
                                                Some(value) => redis::FromRedisValue::from_redis_value(value)
                                                    .map_err(|e| redis::RedisError::from((
                                                        redis::ErrorKind::TypeError,
                                                        "Failed to parse field",
                                                        format!("Field '{}': {}", #field_names, e),
                                                    )))?,
                                                None => return Err(redis::RedisError::from((
                                                    redis::ErrorKind::TypeError,
                                                    "Missing required field",
                                                    #field_names.to_string(),
                                                ))),
                                            }
                                        },
                                    )*
                                })
                            }
                            redis::Value::Nil => {
                                Err(redis::RedisError::from((
                                    redis::ErrorKind::TypeError,
                                    "Cannot deserialize struct from nil value",
                                )))
                            }
                            _ => {
                                Err(redis::RedisError::from((
                                    redis::ErrorKind::TypeError,
                                    "Expected Array or Map for struct",
                                )))
                            }
                        }
                    }
                }
            };

            from_redis_impl.into()
        }
        Fields::Unnamed(fields_unnamed) => {
            let field_count = fields_unnamed.unnamed.len();
            let indices: Vec<syn::Index> = (0..field_count)
                .map(|i| syn::Index::from(i))
                .collect();

            let from_redis_impl = quote! {
                impl redis::FromRedisValue for #type_ident {
                    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                        match v {
                            redis::Value::Array(items) => {
                                if items.len() != #field_count {
                                    return Err(redis::RedisError::from((
                                        redis::ErrorKind::TypeError,
                                        "Array length mismatch",
                                        format!("Expected {} elements, got {}", #field_count, items.len()),
                                    )));
                                }

                                Ok(Self(
                                    #(
                                        redis::FromRedisValue::from_redis_value(&items[#indices])
                                            .map_err(|e| redis::RedisError::from((
                                                redis::ErrorKind::TypeError,
                                                "Failed to parse tuple element",
                                                format!("At index {}: {}", #indices, e),
                                            )))?,
                                    )*
                                ))
                            }
                            redis::Value::Nil => {
                                Err(redis::RedisError::from((
                                    redis::ErrorKind::TypeError,
                                    "Cannot deserialize tuple struct from nil",
                                )))
                            }
                            _ => {
                                Err(redis::RedisError::from((
                                    redis::ErrorKind::TypeError,
                                    "Expected Array for tuple struct",
                                )))
                            }
                        }
                    }
                }
            };

            from_redis_impl.into()
        }
        Fields::Unit => {
            let from_redis_impl = quote! {
                impl redis::FromRedisValue for #type_ident {
                    fn from_redis_value(_v: &redis::Value) -> redis::RedisResult<Self> {
                        Ok(Self)
                    }
                }
            };

            from_redis_impl.into()
        }
    }
}
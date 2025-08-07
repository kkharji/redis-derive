use crate::util::{self, ParsedAttributeMap};
use quote::quote;
use syn::{DataEnum, Fields, Ident};

pub fn derive_to_redis_enum(
    data_enum: DataEnum,
    type_ident: Ident,
    attrs: ParsedAttributeMap,
) -> proc_macro::TokenStream {
    // Check if all variants are unit variants (fieldless)
    let is_unit_enum = data_enum.variants.iter().all(|v| v.fields == Fields::Unit);

    if !is_unit_enum {
        panic!("ToRedisArgs can only be derived for enums with unit variants (no fields). Consider using a struct with an enum field instead.");
    }

    let variant_data: Vec<_> = data_enum
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_name = util::transform_variant_name(
                &variant_ident.to_string(),
                attrs.rename_all.as_ref(),
            );
            (variant_ident, variant_name)
        })
        .collect();

    let variant_matches: Vec<_> = variant_data
        .iter()
        .map(|(variant_ident, variant_name)| {
            quote! {
                #type_ident::#variant_ident => out.write_arg(#variant_name.as_bytes()),
            }
        })
        .collect();

    let to_redis_impl = quote! {
        impl redis::ToRedisArgs for #type_ident {
            fn write_redis_args<W: ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                match self {
                    #(#variant_matches)*
                }
            }

            fn num_of_args(&self) -> usize {
                1 // Enums are always single-argument (the variant name)
            }
        }
    };

    to_redis_impl.into()
}

pub fn derive_from_redis_enum(
    data_enum: DataEnum,
    type_ident: Ident,
    attrs: ParsedAttributeMap,
) -> proc_macro::TokenStream {
    // Check if all variants are unit variants (fieldless)
    let is_unit_enum = data_enum.variants.iter().all(|v| v.fields == Fields::Unit);

    if !is_unit_enum {
        panic!("FromRedisValue can only be derived for enums with unit variants (no fields). Consider using a struct with an enum field instead.");
    }

    let variant_data: Vec<_> = data_enum
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_name = util::transform_variant_name(
                &variant_ident.to_string(),
                attrs.rename_all.as_ref(),
            );
            (variant_ident, variant_name)
        })
        .collect();

    let match_arms: Vec<_> = variant_data
        .iter()
        .map(|(variant_ident, variant_name)| {
            quote! {
                #variant_name => Ok(#type_ident::#variant_ident),
            }
        })
        .collect();

    let variant_names: Vec<&str> = variant_data.iter().map(|(_, name)| name.as_str()).collect();
    let variant_list = variant_names.join(", ");

    // Helper function to create error for unknown variants
    let create_unknown_variant_error = quote! {
        |unknown: &str| -> redis::RedisError {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Unknown enum variant",
                format!(
                    "Unknown variant '{}' for {}. Valid variants: [{}]",
                    unknown,
                    stringify!(#type_ident),
                    #variant_list
                ),
            ))
        }
    };

    // Helper function to parse string to enum
    let parse_string_to_enum = quote! {
        |s: &str| -> redis::RedisResult<#type_ident> {
            let create_error = #create_unknown_variant_error;
            match s {
                #(#match_arms)*
                unknown => Err(create_error(unknown))
            }
        }
    };

    let from_redis_impl = quote! {
        impl redis::FromRedisValue for #type_ident {
            fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
                let parse_str = #parse_string_to_enum;

                match v {
                    // Handle binary string data (most common for stored values)
                    redis::Value::BulkString(data) => {
                        let s = String::from_utf8(data.clone())
                            .map_err(|e| redis::RedisError::from((
                                redis::ErrorKind::TypeError,
                                "Invalid UTF-8 in enum value",
                                e.to_string(),
                            )))?;
                        parse_str(&s)
                    }
                    
                    // Handle simple string responses
                    redis::Value::SimpleString(s) => {
                        parse_str(s)
                    }
                    
                    // Handle verbatim strings (Redis 6+ feature)
                    redis::Value::VerbatimString { text, .. } => {
                        parse_str(text)
                    }
                    
                    // Handle nil values with clear error
                    redis::Value::Nil => {
                        Err(redis::RedisError::from((
                            redis::ErrorKind::TypeError,
                            "Cannot deserialize enum from nil value",
                            format!("Expected string value for {}, got nil", stringify!(#type_ident)),
                        )))
                    }
                    
                    // Handle all other unsupported types
                    _ => {
                        Err(redis::RedisError::from((
                            redis::ErrorKind::TypeError,
                            "Cannot deserialize enum from Redis value type",
                            format!(
                                "Expected string value for {}, got unsupported Redis value type",
                                stringify!(#type_ident)
                            ),
                        )))
                    }
                }
            }
        }
    };

    from_redis_impl.into()
}
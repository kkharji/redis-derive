use super::{DeriveFromRedisArgs, DeriveToRedisArgs};

use quote::quote;
use syn::{DataEnum, Fields, Ident};

impl DeriveToRedisArgs for DataEnum {
    fn derive_to_redis(&self, type_ident: Ident) -> proc_macro::TokenStream {
        let is_unit = self.variants.iter().all(|v| v.fields == Fields::Unit);
        if !is_unit {
            panic!("Only Enums without fields are supported");
        }

        let variant_names = self.variants.iter().map(|variant| &variant.ident);

        let match_arms = variant_names.map(|variant_name| {
            quote! {
                #type_ident::#variant_name => {
                    out.write_arg(stringify!(#variant_name).as_bytes());
                }
            }
        });

        quote! {
            impl redis::ToRedisArgs for #type_ident {
                fn write_redis_args<W: ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                    match self { #(#match_arms),* }
                }
            }
        }
        .into()
    }
}

impl DeriveFromRedisArgs for DataEnum {
    fn derive_from_redis(&self, type_ident: Ident) -> proc_macro::TokenStream {
        let is_unit = self.variants.iter().all(|v| v.fields == Fields::Unit);
        if !is_unit {
            panic!("Only Enums without fields are supported");
        }

        let match_arms = self.variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            quote! {
                stringify!(#variant_name) => Ok(#type_ident::#variant_name),
            }
        });

        let variants_str = self
            .variants
            .iter()
            .map(|variant| variant.ident.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        quote! {
            impl redis::FromRedisValue for #type_ident {
                fn from_redis_value(v: &redis::Value) -> Result<Self, redis::RedisError> {
                    match v {
                        redis::Value::Data(s) => match std::str::from_utf8(&s[..])? {
                            #(#match_arms)*
                            v => Err(redis::RedisError::from((
                                redis::ErrorKind::TypeError,
                                "Invalid enum variant:",
                                format!("{}, Expected one of: {}", v, #variants_str),
                            ))),
                        },
                        v => Err(redis::RedisError::from((
                            redis::ErrorKind::TypeError,
                            "Expected Redis string, got:", format!("{:?}", v)
                        ))),
                    }
                }
            }
        }
        .into()
    }
}

use crate::util::{self, ParsedAttributeMap};
use crate::{DeriveFromRedisArgs, DeriveToRedisArgs};

use quote::quote;
use syn::{self, DataEnum, Fields, Ident};

impl DeriveToRedisArgs for DataEnum {
    fn derive_to_redis(
        &self,
        type_ident: Ident,
        attrs: ParsedAttributeMap,
    ) -> proc_macro::TokenStream {
        let is_unit = self.variants.iter().all(|v| v.fields == Fields::Unit);
        if !is_unit {
            panic!("Only Enums without fields are supported");
        }

        let variant_names = self.variants.iter().map(|v| &v.ident).collect::<Vec<_>>();
        let rename_all = attrs.get("rename_all").map(|v| v.as_str());

        let match_arms = variant_names
            .iter()
            .map(|v| (v, util::transform_variant(&v.to_string(), rename_all)))
            .map(|(name, value)| {
                quote! {
                    #type_ident::#name => {
                        out.write_arg(#value.as_bytes());
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
    fn derive_from_redis(
        &self,
        type_ident: Ident,
        attrs: ParsedAttributeMap,
    ) -> proc_macro::TokenStream {
        let is_unit = self.variants.iter().all(|v| v.fields == Fields::Unit);
        if !is_unit {
            panic!("Only Enums without fields are supported");
        }

        let rename_all = attrs.get("rename_all").map(|v| v.as_str());
        let (variants_str, match_arms): (Vec<_>, Vec<_>) = self
            .variants
            .iter()
            .map(|v| {
                (
                    &v.ident,
                    util::transform_variant(&v.ident.to_string(), rename_all),
                )
            })
            .map(|(ident, value)| {
                (
                    value.clone(),
                    quote! {
                        #value => Ok(#type_ident::#ident),
                    },
                )
            })
            .unzip();

        let variants_str = variants_str.join(", ");

        quote! {
            impl redis::FromRedisValue for #type_ident {
                fn from_redis_value(v: &redis::Value) -> Result<Self, redis::RedisError> {
                    use redis::{ErrorKind::TypeError, Value};

                    let Value::Data(data) = v else {
                        let msg = format!("{:?}", v);
                        return Err((TypeError, "Expected Redis string, got:", msg).into());
                    };

                    let value = std::str::from_utf8(&data[..])?;

                    match value {
                        #(#match_arms)*
                        v => {
                            let msg = format!("{}, Expected one of: {}", v, #variants_str);
                            Err((TypeError, "Invalid enum variant:", msg).into())
                        },
                    }
                }
            }
        }
        .into()
    }
}

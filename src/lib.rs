
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data::Struct, DataStruct, Fields, FieldsNamed};


#[proc_macro_derive(ToRedisArgs)]
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
pub fn from_redis_value(tokenstream : TokenStream) -> TokenStream {
    let abstract_syntax_tree = parse_macro_input!(tokenstream as DeriveInput);
    let struct_idententifier = abstract_syntax_tree.ident;
    let (field_idententifier_strs, field_idententifiers) = derive_fields(abstract_syntax_tree.data);

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
                                    #field_idententifiers : redis::from_redis_value(
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
                            "the data return from the redis database was not in the bulk dataformat or the length of the bulk data is not devisable by two"))
                    )
                }
            }
        }
    }.into()
}

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

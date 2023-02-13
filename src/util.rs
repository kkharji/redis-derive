use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use std::collections::HashMap;
use syn::{Attribute, Meta, NestedMeta};

pub type ParsedAttributeMap = HashMap<String, String>;

/// Parses the Redis attributes in the given list of attributes, and returns a
/// mapping of attribute names to their string values.
pub fn parse_attributes(attributes: &[Attribute]) -> ParsedAttributeMap {
    let mut attr_map = HashMap::new();
    for attribute in attributes {
        if !attribute.path.is_ident("redis") {
            continue;
        }

        if let Ok(Meta::List(meta)) = attribute.parse_meta() {
            if meta.path.is_ident("redis") {
                for nested_meta in meta.nested {
                    if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta {
                        let attr_name = name_value
                            .path
                            .get_ident()
                            .expect("Attribute name expected")
                            .to_string();
                        let attr_value = match &name_value.lit {
                            syn::Lit::Str(lit_str) => lit_str.value(),
                            _ => panic!("Attribute value must be a string literal"),
                        };
                        attr_map.insert(attr_name, attr_value);
                    }
                }
            }
        }
    }
    attr_map
}

/// Transforms a variant value into the desired case style based on the provided `rename_all` option.
///
/// This function panics if an invalid `rename_all` value is provided.
pub fn transform_variant(variant_value: &str, rename_all: Option<&str>) -> String {
    let renamed = rename_all.map(|rule| match rule {
        "lowercase" => variant_value.to_lowercase(),
        "UPPERCASE" => variant_value.to_uppercase(),
        "PascalCase" => variant_value.to_pascal_case(),
        "camelCase" => variant_value.to_lower_camel_case(),
        "snake_case" => variant_value.to_snake_case(),
        "kebab-case" => variant_value.to_kebab_case(),
        _ => panic!("Invalid rename_all value"),
    });

    renamed.unwrap_or_else(|| variant_value.to_string())
}

use syn::{Attribute, Meta};

#[derive(Debug, Default, Clone)]
pub struct ParsedAttributeMap {
    pub rename_all: Option<String>,
    pub cluster_key: Option<String>,
    pub cache: bool,
    pub ttl: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct FieldAttributes {
    pub skip: bool,
    pub rename: Option<String>,
    pub expire: Option<String>,
    pub expire_at: Option<String>,
}

pub fn parse_attributes(attrs: &[Attribute]) -> ParsedAttributeMap {
    let mut parsed = ParsedAttributeMap::default();

    for attr in attrs {
        if !attr.path().is_ident("redis") {
            continue;
        }

        // Parse #[redis(...)] attributes
        if let Meta::List(list) = &attr.meta {
            // Convert token stream to string and parse manually for now
            let tokens_str = list.tokens.to_string();
            
            // Look for rename_all = "value"
            if let Some(rename_all_value) = extract_quoted_value(&tokens_str, "rename_all") {
                parsed.rename_all = Some(rename_all_value);
            }
            
            // Look for cluster_key = "value"  
            if let Some(cluster_key_value) = extract_quoted_value(&tokens_str, "cluster_key") {
                parsed.cluster_key = Some(cluster_key_value);
            }
            
            // Look for ttl = "value"
            if let Some(ttl_value) = extract_quoted_value(&tokens_str, "ttl") {
                parsed.ttl = Some(ttl_value);
            }
            
            // Look for cache (boolean flag)
            if tokens_str.contains("cache") {
                parsed.cache = true;
            }
        }
    }

    parsed
}

pub fn parse_field_attributes(attrs: &[Attribute]) -> FieldAttributes {
    let mut field_attrs = FieldAttributes::default();

    for attr in attrs {
        if !attr.path().is_ident("redis") {
            continue;
        }

        if let Meta::List(list) = &attr.meta {
            let tokens_str = list.tokens.to_string();
            
            if tokens_str.contains("skip") {
                field_attrs.skip = true;
            }
            
            if let Some(rename_value) = extract_quoted_value(&tokens_str, "rename") {
                field_attrs.rename = Some(rename_value);
            }
            
            if let Some(expire_value) = extract_quoted_value(&tokens_str, "expire") {
                // Make sure it's not expire_at
                if !tokens_str.contains("expire_at") {
                    field_attrs.expire = Some(expire_value);
                }
            }
            
            if let Some(expire_at_value) = extract_quoted_value(&tokens_str, "expire_at") {
                field_attrs.expire_at = Some(expire_at_value);
            }
        }
    }

    field_attrs
}

/// Extract a quoted string value from tokens like: key = "value"
fn extract_quoted_value(tokens: &str, key: &str) -> Option<String> {
    // Look for pattern: key = "value"
    let pattern = format!("{} =", key);
    if let Some(start_pos) = tokens.find(&pattern) {
        let after_equals = &tokens[start_pos + pattern.len()..];
        
        // Find the opening quote
        if let Some(quote_start) = after_equals.find('"') {
            let after_quote = &after_equals[quote_start + 1..];
            
            // Find the closing quote
            if let Some(quote_end) = after_quote.find('"') {
                return Some(after_quote[..quote_end].to_string());
            }
        }
    }
    None
}

pub fn transform_variant_name(variant_name: &str, rename_all: Option<&String>) -> String {
    let rename_rule = match rename_all {
        Some(rule) => rule.as_str(),
        None => return variant_name.to_string(),
    };

    match rename_rule {
        "lowercase" => variant_name.to_lowercase(),
        "UPPERCASE" => variant_name.to_uppercase(),
        "PascalCase" => to_pascal_case(variant_name),
        "camelCase" => to_camel_case(variant_name),
        "snake_case" => to_snake_case(variant_name),
        "kebab-case" => to_kebab_case(variant_name),
        _ => {
            panic!(
                "Invalid rename_all value: {rename_rule}. Valid options: lowercase, UPPERCASE, PascalCase, camelCase, snake_case, kebab-case"
            );
        }
    }
}

pub fn transform_field_name(
    field_name: &str,
    rename_all: Option<&String>,
    field_rename: Option<&String>,
) -> String {
    // Field-level rename takes precedence
    if let Some(rename) = field_rename {
        return rename.clone();
    }

    transform_variant_name(field_name, rename_all)
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.extend(c.to_lowercase());
        }
    }

    result
}

fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    if pascal.is_empty() {
        return pascal;
    }

    let mut chars = pascal.chars();
    let first_char = chars.next().unwrap().to_lowercase().to_string();
    first_char + &chars.collect::<String>()
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_lower = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase()
            && i > 0
            && (prev_is_lower || s.chars().nth(i + 1).is_some_and(|next| next.is_lowercase()))
        {
            result.push('_');
        }
        result.extend(c.to_lowercase());
        prev_is_lower = c.is_lowercase();
    }

    result
}

fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_transformations() {
        assert_eq!(to_snake_case("MyFieldName"), "my_field_name");
        assert_eq!(to_pascal_case("my_field_name"), "MyFieldName");
        assert_eq!(to_camel_case("my_field_name"), "myFieldName");
        assert_eq!(to_kebab_case("MyFieldName"), "my-field-name");
    }

    #[test]
    fn test_transform_variant_name() {
        assert_eq!(
            transform_variant_name("InProgress", Some(&"snake_case".to_string())),
            "in_progress"
        );
        assert_eq!(
            transform_variant_name("InProgress", Some(&"kebab-case".to_string())),
            "in-progress"
        );
        assert_eq!(
            transform_variant_name("InProgress", Some(&"lowercase".to_string())),
            "inprogress"
        );
        assert_eq!(transform_variant_name("InProgress", None), "InProgress");
    }

    #[test]
    fn test_extract_quoted_value() {
        assert_eq!(
            extract_quoted_value(r#"rename_all = "snake_case""#, "rename_all"),
            Some("snake_case".to_string())
        );
        assert_eq!(
            extract_quoted_value(r#"expire = "3600""#, "expire"),
            Some("3600".to_string())
        );
        assert_eq!(extract_quoted_value("cache", "cache"), None);
    }
}
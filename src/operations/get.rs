use toml::Value;
use serde_json;

pub fn get_value(toml_value: &Value, key: &str) -> String {
    let mut current = toml_value;
    for part in key.split('.') {
        match navigate_value(current, part) {
            Some(value) => current = value,
            None => return String::new(),
        }
    }
    format_value(current)
}

fn navigate_value<'a>(current: &'a Value, part: &str) -> Option<&'a Value> {
    if let Some(bracket_pos) = part.find('[') {
        let key = &part[..bracket_pos];
        let array_part = &part[bracket_pos..];
        navigate_table(current, key).and_then(|v| navigate_nested_array(v, array_part))
    } else {
        navigate_table(current, part)
    }
}

fn navigate_table<'a>(current: &'a Value, key: &str) -> Option<&'a Value> {
    current.get(key)
}

fn navigate_nested_array<'a>(current: &'a Value, key: &str) -> Option<&'a Value> {
    let mut value = current;
    let mut remaining = key;

    while let Some(close_bracket) = remaining.find(']') {
        let index: usize = remaining[1..close_bracket].parse().ok()?;
        value = value.as_array()?.get(index)?;
        
        remaining = &remaining[close_bracket + 1..];
        if remaining.starts_with('[') {
            continue;
        } else if remaining.is_empty() {
            break;
        } else {
            return None;
        }
    }

    Some(value)
}

fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Datetime(dt) => dt.to_string(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_toml() -> Value {
        toml::from_str(r#"
            [foo]
            bar = "some string"
            colors = ["red", "yellow", "green"]
            nested_arrays_of_ints = [[1, 2], [3, 4, 5]]
            name = { first = "Tom", last = "Preston-Werner" }

            [fruits]
            apples = 3
            bananas = 5

            [[products]]
            name = "Hammer"
            sku = 738594937

            [[products]]
            name = "Nail"
            sku = 284758393

            [deep]
            nested.value = 42
        "#).unwrap()
    }

    #[test]
    fn test_get_simple_values() {
        let toml_value = create_sample_toml();

        assert_eq!(get_value(&toml_value, "foo.bar"), "some string");
        assert_eq!(get_value(&toml_value, "fruits.apples"), "3");
        assert_eq!(get_value(&toml_value, "deep.nested.value"), "42");
    }

    #[test]
    fn test_get_array_values() {
        let toml_value = create_sample_toml();

        assert_eq!(get_value(&toml_value, "foo.colors"), "[\"red\",\"yellow\",\"green\"]");
        assert_eq!(get_value(&toml_value, "foo.colors[1]"), "yellow");
        assert_eq!(get_value(&toml_value, "foo.colors[3]"), "");
    }

    #[test]
    fn test_get_nested_array_values() {
        let toml_value = create_sample_toml();

        assert_eq!(get_value(&toml_value, "foo.nested_arrays_of_ints[0]"), "[1,2]");
        assert_eq!(get_value(&toml_value, "foo.nested_arrays_of_ints[0][0]"), "1");
        assert_eq!(get_value(&toml_value, "foo.nested_arrays_of_ints[1][1]"), "4");
        assert_eq!(get_value(&toml_value, "foo.nested_arrays_of_ints[2][0]"), "");
    }

    #[test]
    fn test_get_table_values() {
        let toml_value = create_sample_toml();

        assert_eq!(get_value(&toml_value, "foo.name"), "{\"first\":\"Tom\",\"last\":\"Preston-Werner\"}");
        assert_eq!(get_value(&toml_value, "foo.name.first"), "Tom");
        assert_eq!(get_value(&toml_value, "fruits"), "{\"apples\":3,\"bananas\":5}");
    }

    #[test]
    fn test_get_array_of_tables() {
        let toml_value = create_sample_toml();

        assert_eq!(get_value(&toml_value, "products"), "[{\"name\":\"Hammer\",\"sku\":738594937},{\"name\":\"Nail\",\"sku\":284758393}]");
        assert_eq!(get_value(&toml_value, "products[1]"), "{\"name\":\"Nail\",\"sku\":284758393}");
        assert_eq!(get_value(&toml_value, "products[1].sku"), "284758393");
        assert_eq!(get_value(&toml_value, "products[1].name"), "Nail");
        assert_eq!(get_value(&toml_value, "products[2].name"), "");
    }

    #[test]
    fn test_get_nonexistent_values() {
        let toml_value = create_sample_toml();

        assert_eq!(get_value(&toml_value, "nonexistent"), "");
        assert_eq!(get_value(&toml_value, "foo.nonexistent"), "");
        assert_eq!(get_value(&toml_value, "foo.colors[10]"), "");
        assert_eq!(get_value(&toml_value, "foo.nested_arrays_of_ints[5][1]"), "");
        assert_eq!(get_value(&toml_value, "deep.nested.nonexistent"), "");
    }
}

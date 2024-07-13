use toml::{Value, Table};
use super::parse::parse_value;
use std::error::Error;

pub fn set_value(toml_value: &mut Value, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
    set_value_recursive(toml_value, key, value)
}

fn set_value_recursive(current: &mut Value, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
    let (current_part, remaining) = split_first_key(key);
    match parse_key_part(current_part) {
        KeyPart::Normal(key) => handle_normal_key(current, key, remaining, value),
        KeyPart::Array(key, indices) => {
            if indices.is_empty() && value.trim().starts_with("...") {
                handle_array_spread(current, key, value)
            } else {
                handle_array_key(current, key, indices, remaining, value)
            }
        },
    }
}

fn handle_array_spread(current: &mut Value, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
    ensure_table(current);
    let table = current.as_table_mut().unwrap();
    if !table.contains_key(key) {
        table.insert(key.to_string(), Value::Array(Vec::new()));
    }
    let array = table.get_mut(key).unwrap().as_array_mut().unwrap();
    let parsed_value = parse_value(value)?;
    if let Value::Array(inner_array) = parsed_value {
        array.extend(inner_array);
    } else {
        return Err("Expected an array after spread operator".into());
    }
    Ok(())
}

fn handle_normal_key(current: &mut Value, key: &str, remaining: &str, value: &str) -> Result<(), Box<dyn Error>> {
    ensure_table(current);
    let table = current.as_table_mut().unwrap();
    if !table.contains_key(key) {
        table.insert(key.to_string(), Value::Table(Table::new()));
    }
    let next = table.get_mut(key).unwrap();
    if remaining.is_empty() {
        *next = parse_value(value)?;
    } else {
        set_value_recursive(next, remaining, value)?;
    }
    Ok(())
}

fn handle_array_key(current: &mut Value, key: &str, indices: Vec<Option<usize>>, remaining: &str, value: &str) -> Result<(), Box<dyn Error>> {
    ensure_table(current);
    let table = current.as_table_mut().unwrap();
    if !table.contains_key(key) {
        table.insert(key.to_string(), Value::Array(Vec::new()));
    }
    let mut current_value = table.get_mut(key).unwrap();

    for (i, index_opt) in indices.iter().enumerate() {
        match index_opt {
            Some(index) => {
                current_value = handle_specific_index(current_value, *index)?;
            },
            None => {
                handle_append_index(current_value, i == indices.len() - 1 && remaining.is_empty(), value)?;
                return Ok(());  // We're done after appending
            },
        };
    }
    

    // Only set the value if it's not a spread operation and there's no remaining key
    if remaining.is_empty() && !value.trim().starts_with("...") {
        *current_value = parse_value(value)?;
    } else if !remaining.is_empty() {
        set_value_recursive(current_value, remaining, value)?;
    }
    Ok(())
}


fn handle_specific_index(value: &mut Value, index: usize) -> Result<&mut Value, Box<dyn Error>> {
    ensure_array(value);
    let array = value.as_array_mut().unwrap();
    if index >= array.len() {
        array.resize_with(index + 1, || Value::Array(Vec::new()));
    }
    Ok(&mut array[index])
}

fn handle_append_index(value: &mut Value, is_last: bool, new_value: &str) -> Result<(), Box<dyn Error>> {
    
    ensure_array(value);
    let array = value.as_array_mut().unwrap();
    
    if is_last {
        if new_value.trim().starts_with("...") {
            let parsed_value = parse_value(&new_value[3..])?;
            if let Value::Array(inner_array) = parsed_value {
                array.extend(inner_array);
            } else {
                return Err("Expected an array after spread operator".into());
            }
        } else {
            let parsed_value = parse_value(new_value)?;
            array.push(parsed_value);
        }
    } else {
        array.push(Value::Array(Vec::new()));
    }

    Ok(())
}


enum KeyPart<'a> {
    Normal(&'a str),
    Array(&'a str, Vec<Option<usize>>),
}

fn parse_key_part(part: &str) -> KeyPart {
    if let Some(bracket_pos) = part.find('[') {
        let key = &part[..bracket_pos];
        let indices = parse_indices(&part[bracket_pos..]);
        KeyPart::Array(key, indices)
    } else {
        KeyPart::Normal(part)
    }
}

fn parse_indices(s: &str) -> Vec<Option<usize>> {
    let mut indices = Vec::new();
    let mut current_index = String::new();
    let mut in_bracket = false;

    for c in s.chars() {
        match c {
            '[' => in_bracket = true,
            ']' => {
                in_bracket = false;
                indices.push(current_index.parse().ok());
                current_index.clear();
            },
            _ if in_bracket => current_index.push(c),
            _ => {}
        }
    }
    indices
}

fn split_first_key(key: &str) -> (&str, &str) {
    let mut depth = 0;
    for (i, c) in key.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => depth -= 1,
            '.' if depth == 0 => return (&key[..i], &key[i+1..]),
            _ => {}
        }
    }
    (key, "")
}

fn ensure_table(value: &mut Value) {
    if !value.is_table() {
        *value = Value::Table(Table::new());
    }
}

fn ensure_array(value: &mut Value) {
    if !value.is_array() {
        *value = Value::Array(Vec::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::get::get_value;

    fn create_sample_toml() -> Value {
        toml::from_str(r#"
            [foo]
            bar = "some_value"
            integers = [1, 2, 3]
            
            [[products]]
            name = "Hammer"
            sku = 738594937

            [[products]]
            name = "Nail"
            sku = 284758393
        "#).unwrap()
    }

    #[test]
    fn test_set_simple_values() {
        let mut toml_value = create_sample_toml();

        set_value(&mut toml_value, "foo.bar", "new_value").unwrap();
        assert_eq!(get_value(&toml_value, "foo.bar"), "new_value");

        set_value(&mut toml_value, "new.key", "new_value").unwrap();
        assert_eq!(get_value(&toml_value, "new.key"), "new_value");
    }

    #[test]
    fn test_set_array_values() {
        let mut toml_value = create_sample_toml();

        set_value(&mut toml_value, "foo.integers[1]", "5").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers[1]"), "5");

        set_value(&mut toml_value, "foo.integers[]", "4").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers[3]"), "4");

        set_value(&mut toml_value, "foo.integers[]", "[5, 6]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers[4]"), "[5,6]");
    }

    #[test]
    fn test_set_array_values_with_spread() {
        let mut toml_value = create_sample_toml();

        set_value(&mut toml_value, "foo.integers[]", "...[4, 5, 6]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers"), "[1,2,3,4,5,6]");

        set_value(&mut toml_value, "new_array[]", "...[1, 2, 3]").unwrap();
        assert_eq!(get_value(&toml_value, "new_array"), "[1,2,3]");
    }

    #[test]
    fn test_set_nested_values() {
        let mut toml_value = create_sample_toml();

        set_value(&mut toml_value, "products[1].name", "Screw").unwrap();
        assert_eq!(get_value(&toml_value, "products[1].name"), "Screw");

        set_value(&mut toml_value, "products[2].name", "Bolt").unwrap();
        assert_eq!(get_value(&toml_value, "products[2].name"), "Bolt");

        set_value(&mut toml_value, "deep.nested.value", "42").unwrap();
        assert_eq!(get_value(&toml_value, "deep.nested.value"), "42");
    }

    #[test]
    fn test_set_complex_nested_arrays() {
        let mut toml_value = create_sample_toml();

        set_value(&mut toml_value, "complex[0][1][2]", "nested").unwrap();
        assert_eq!(get_value(&toml_value, "complex[0][1][2]"), "nested");

        set_value(&mut toml_value, "complex[1][]", "value").unwrap();
        assert_eq!(get_value(&toml_value, "complex[1][0]"), "value");
    }
}
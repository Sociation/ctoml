use toml::Value;
use toml::Table;

pub fn remove_value(toml_value: &mut Value, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = key.split('.').collect();
    remove_recursive(toml_value, &parts)
}    

fn remove_recursive(value: &mut Value, parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    if parts.is_empty() {
        return Ok(());
    }

    let part = parts[0];

    match value {
        Value::Table(table) => remove_from_table(table, part, &parts[1..]),
        Value::Array(arr) => remove_from_array(arr, part, &parts[1..]),
        _ => Err("Not a table or array".into()),
    }
}

fn remove_from_table(table: &mut Table, part: &str, remaining_parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    if remaining_parts.is_empty() {
        if let Some(bracket_pos) = part.find('[') {
            let key = &part[..bracket_pos];
            if let Some(value) = table.get_mut(key) {
                remove_from_array_recursive(value, &part[bracket_pos..], &[])?;
            }
        } else {
            table.remove(part);
        }
    } else if let Some(next) = table.get_mut(part) {
        remove_recursive(next, remaining_parts)?;
    }
    Ok(())
}

fn remove_from_array(arr: &mut Vec<Value>, part: &str, remaining_parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    remove_from_array_recursive(&mut Value::Array(arr.to_vec()), part, remaining_parts)?;
    if let Value::Array(new_arr) = &mut Value::Array(arr.to_vec()) {
        *arr = new_arr.clone();
    }
    Ok(())
}

fn remove_from_array_recursive(value: &mut Value, part: &str, remaining_parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(close_bracket) = part.find(']') {
        let index_str = &part[1..close_bracket];
        let arr = value.as_array_mut().ok_or("Not an array")?;

        if index_str.is_empty() {
            handle_array_removal(arr, &remaining_parts.join("."))?;
        } else if let Ok(index) = index_str.parse::<usize>() {
            if index < arr.len() {
                if remaining_parts.is_empty() {
                    if part[close_bracket+1..].is_empty() {
                        arr.remove(index);
                    } else {
                        remove_from_array_recursive(&mut arr[index], &part[close_bracket+1..], remaining_parts)?;
                    }
                } else {
                    remove_recursive(&mut arr[index], remaining_parts)?;
                }
            }
        } else {
            handle_array_removal(arr, &format!("{}{}", index_str, remaining_parts.join(".")))?;
        }
    }
    Ok(())
}

fn handle_array_removal(arr: &mut Vec<Value>, index_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    if index_str.is_empty() {
        arr.clear();
    } else if index_str.starts_with(':') {
        let n: usize = index_str[1..].parse()?;
        if n > arr.len() {
            arr.clear();
        } else {
            arr.truncate(arr.len() - n);
        }
    } else if index_str.ends_with(':') {
        let n: usize = index_str[..index_str.len()-1].parse()?;
        if n >= arr.len() {
            arr.clear();
        } else {
            *arr = arr.drain(n..).collect();
        }
    } else {
        let indices: Result<Vec<usize>, _> = index_str.split(',')
            .map(|s| s.trim().parse::<usize>())
            .collect();
        let mut indices = indices?;
        indices.sort_unstable();
        indices.reverse();
        for &index in &indices {
            if index < arr.len() {
                arr.remove(index);
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::get::get_value;

    fn create_sample_toml() -> Value {
        toml::from_str(r#"
            [foo]
            integers = [1, 2, 3, 4, 5]
            nested_arrays_of_ints = [[0, 1], [3, 4, 5]]
            
            [bar]
            name = "test"
            
            [[products]]
            name = "Hammer"
            sku = 738594937

            [[products]]
            name = "Nail"
            sku = 284758393
        "#).unwrap()
    }

    #[test]
    fn test_remove_simple_values() {
        let mut toml_value = create_sample_toml();

        remove_value(&mut toml_value, "foo.integers").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers"), "");

        remove_value(&mut toml_value, "bar.name").unwrap();
        assert_eq!(get_value(&toml_value, "bar.name"), "");

        remove_value(&mut toml_value, "bar").unwrap();
        assert_eq!(get_value(&toml_value, "bar"), "");
    }

    #[test]
    fn test_remove_array_slices() {
        let mut toml_value = create_sample_toml();

        remove_value(&mut toml_value, "foo.integers[:2]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers"), "[1,2,3]");

        let mut toml_value = create_sample_toml();
        remove_value(&mut toml_value, "foo.integers[2:]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers"), "[3,4,5]");
    }

    #[test]
    fn test_remove_array_elements() {
        let mut toml_value = create_sample_toml();

        remove_value(&mut toml_value, "foo.integers[1]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers"), "[1,3,4,5]");

        let mut toml_value = create_sample_toml();
        remove_value(&mut toml_value, "foo.integers[0,2,4]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers"), "[2,4]");
    }

    #[test]
    fn test_remove_nested_array_elements() {
        let mut toml_value = create_sample_toml();

        remove_value(&mut toml_value, "foo.nested_arrays_of_ints[0][:1]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.nested_arrays_of_ints[0]"), "[0]");

        let mut toml_value = create_sample_toml();
        remove_value(&mut toml_value, "foo.nested_arrays_of_ints[1][1:]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.nested_arrays_of_ints[1]"), "[4,5]");

        let mut toml_value = create_sample_toml();
        remove_value(&mut toml_value, "foo.nested_arrays_of_ints[1][0,2]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.nested_arrays_of_ints[1]"), "[4]");
    }

    #[test]
    fn test_remove_array_of_tables() {
        let mut toml_value = create_sample_toml();

        remove_value(&mut toml_value, "products[0]").unwrap();
        assert_eq!(get_value(&toml_value, "products[0].name"), "Nail");

        remove_value(&mut toml_value, "products").unwrap();
        assert_eq!(get_value(&toml_value, "products"), "");
    }

    #[test]
    fn test_remove_nonexistent_values() {
        let mut toml_value = create_sample_toml();

        remove_value(&mut toml_value, "nonexistent").unwrap();
        remove_value(&mut toml_value, "foo.nonexistent").unwrap();
        remove_value(&mut toml_value, "foo.integers[10]").unwrap();
        remove_value(&mut toml_value, "foo.nested_arrays_of_ints[5][1]").unwrap();

        // Ensure the original structure is unchanged
        assert_eq!(get_value(&toml_value, "foo.integers"), "[1,2,3,4,5]");
        assert_eq!(get_value(&toml_value, "foo.nested_arrays_of_ints"), "[[0,1],[3,4,5]]");
    }

    #[test]
    fn test_remove_array_slices_edge_cases() {
        let mut toml_value = create_sample_toml();

        remove_value(&mut toml_value, "foo.integers[:10]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers"), "[]");

        let mut toml_value = create_sample_toml();
        remove_value(&mut toml_value, "foo.integers[10:]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers"), "[]");

        let mut toml_value = create_sample_toml();
        remove_value(&mut toml_value, "foo.integers[:0]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers"), "[1,2,3,4,5]");

        let mut toml_value = create_sample_toml();
        remove_value(&mut toml_value, "foo.integers[0:]").unwrap();
        assert_eq!(get_value(&toml_value, "foo.integers"), "[1,2,3,4,5]");
    }
}
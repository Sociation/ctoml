use toml::Value;

pub fn parse_value(value: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let trimmed = value.trim();
    
    if trimmed.starts_with("...") {
        return parse_array(&trimmed[3..]);
    }

    if let Ok(v) = trimmed.parse::<i64>() {
        return Ok(Value::Integer(v));
    }

    if let Ok(v) = trimmed.parse::<f64>() {
        return Ok(Value::Float(v));
    }

    if let Ok(v) = trimmed.parse::<bool>() {
        return Ok(Value::Boolean(v));
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return parse_table(trimmed);
    }

    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return parse_array(trimmed);
    }

    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        return Ok(Value::String(trimmed[1..trimmed.len()-1].to_string()));
    }

    Ok(Value::String(trimmed.to_string()))
}

fn parse_table(value: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut table = toml::Table::new();
    let inner = &value[1..value.len()-1];
    
    if inner.trim().is_empty() {
        return Ok(Value::Table(table));
    }

    for pair in inner.split(',') {
        let mut kv = pair.splitn(2, ':');
        let key = kv.next().ok_or("Missing key")?.trim();
        let val = kv.next().ok_or("Missing value")?.trim();
        table.insert(key.to_string(), parse_value(val)?);
    }
    Ok(Value::Table(table))
}

fn parse_array(value: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut current = String::new();

    let inner = value.trim();

    if !inner.starts_with('[') || !inner.ends_with(']') {
        return Err("Invalid array format".into());
    }

    let inner = &inner[1..inner.len()-1];
    
    if inner.trim().is_empty() {
        return Ok(Value::Array(result));
    }

    for c in inner.chars() {
        match c {
            '[' => {
                depth += 1;
                current.push(c);
            }
            ']' => {
                if depth == 0 {
                    return Err("Unbalanced brackets in array".into());
                }
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                if !current.is_empty() {
                    result.push(parse_value(current.trim())?);
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }

    if depth != 0 {
        return Err("Unbalanced brackets in array".into());
    }

    if !current.is_empty() {
        result.push(parse_value(current.trim())?);
    }

    Ok(Value::Array(result))
}


#[cfg(test)]
mod tests {
    use super::*;
    use toml::Table;

    #[test]
    fn test_parse_simple_values() {
        assert_eq!(parse_value("42").unwrap(), Value::Integer(42));
        assert_eq!(parse_value("3.14").unwrap(), Value::Float(3.14));
        assert_eq!(parse_value("true").unwrap(), Value::Boolean(true));
        assert_eq!(parse_value("false").unwrap(), Value::Boolean(false));
        assert_eq!(parse_value("\"hello\"").unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_parse_arrays() {
        assert_eq!(parse_value("[1, 2, 3]").unwrap(), Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3)
        ]));
        assert_eq!(parse_value("[\"a\", \"b\", \"c\"]").unwrap(), Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string())
        ]));
    }

    #[test]
    fn test_parse_nested_arrays() {
        assert_eq!(parse_value("[[1, 2], [3, 4]]").unwrap(), Value::Array(vec![
            Value::Array(vec![Value::Integer(1), Value::Integer(2)]),
            Value::Array(vec![Value::Integer(3), Value::Integer(4)])
        ]));
    }

    #[test]
    fn test_parse_tables() {
        let parsed = parse_value("{key1: \"value1\", key2: 42}").unwrap();
        if let Value::Table(table) = parsed {
            assert_eq!(table.get("key1"), Some(&Value::String("value1".to_string())));
            assert_eq!(table.get("key2"), Some(&Value::Integer(42)));
        } else {
            panic!("Expected a table");
        }
    }

    #[test]
    fn test_parse_empty_structures() {
        assert_eq!(parse_value("[]").unwrap(), Value::Array(vec![]));
        assert_eq!(parse_value("{}").unwrap(), Value::Table(Table::new()));
    }

    #[test]
    fn test_parse_whitespace_handling() {
        assert_eq!(parse_value(" 42 ").unwrap(), Value::Integer(42));
        assert_eq!(parse_value("\t[1, 2, 3]\n").unwrap(), Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3)
        ]));
    }

    #[test]
    fn test_parse_invalid_input() {
        assert_eq!(parse_value("[1, 2, 3").unwrap(), Value::String("[1, 2, 3".to_string()));
        assert_eq!(parse_value("{key: value").unwrap(), Value::String("{key: value".to_string()));
    }
}
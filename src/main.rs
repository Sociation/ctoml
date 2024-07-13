use std::env;
use std::fs;
use std::path::Path;
use toml::Value;

mod operations;

fn print_usage() {
    eprintln!("Usage: toml [OPTION] PATH KEY [VALUE]");
    eprintln!("Manipulate TOML files from the command line.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -r, --remove    Remove the specified key");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  PATH            Path to the TOML file");
    eprintln!("  KEY             Key to read, write, or remove (use dot notation for nested keys)");
    eprintln!("  VALUE           Value to write (required for write operations)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  toml config.toml app.name");
    eprintln!("  toml config.toml app.version 1.0.0");
    eprintln!("  toml config.toml database.ports[] 5432");
    eprintln!("  toml -r config.toml app.deprecated_field");
    eprintln!();
    eprintln!("For more information, visit: https://github.com/sociation/ctoml");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let (remove, arg_index) = if args[1] == "-r" || args[1] == "--remove" {
        (true, 2)
    } else {
        (false, 1)
    };

    let path = &args[arg_index];
    let key = &args[arg_index + 1];
    let value = args.get(arg_index + 2);

    let mut toml_content = if Path::new(path).exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };

    let mut toml_value: Value = if toml_content.is_empty() {
        Value::Table(toml::map::Map::new())
    } else {
        toml::from_str(&toml_content)?
    };

    if remove {
        operations::remove_value(&mut toml_value, key)?;
    } else if let Some(val) = value {
        operations::set_value(&mut toml_value, key, val)?;
    } else {
        let result = operations::get_value(&toml_value, key);
        println!("{}", result);
        return Ok(());
    }

    toml_content = toml::to_string(&toml_value)?;
    fs::write(path, toml_content)?;

    Ok(())
}

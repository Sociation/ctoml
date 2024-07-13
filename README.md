# CTOML Editor

A command-line interface tool for reading, writing, and removing values from TOML files.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Reading Values](#reading-values)
  - [Writing Values](#writing-values)
  - [Removing Values](#removing-values)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Read**: Retrieve values from TOML files using dot notation and array indexing.
- **Write**: Modify or add new values to TOML files, supporting nested structures and arrays.
- **Remove**: Delete keys or array elements from TOML files.
- **Support for complex data structures**: Handle nested tables, arrays, and mixed data types.
- **Flexible syntax**: Use dot notation for nested keys and bracket notation for array indexing.

## Installation

To install the TOML CLI Editor, you need to have Rust and Cargo installed on your system. If you don't have them, you can install them from [rustup.rs](https://rustup.rs/).

Once you have Rust and Cargo set up, you can install the TOML CLI Editor using:

```bash
cargo install ctoml
```

## Usage

The general syntax for using the TOML CLI Editor is:

```bash
toml <file> <key> [value]
```

For removing values:

```bash
toml -r <file> <key>
```

Where:
- `<file>` is the path to your TOML file
- `<key>` is the key you want to operate on (using dot notation for nested keys and brackets for array indexing)
- `[value]` is the new value (only for writing operations)

### Reading Values

To read a value from the TOML file:

```bash
toml <file> <key>
```

### Writing Values

To write a value to the TOML file:

```bash
toml <file> <key> <value>
```

### Removing Values

To remove a value from the TOML file:

```bash
toml -r <file> <key>
```

## Examples

Assuming we have a `sample.toml` file with the following content:

```toml
food.snickers.taste.sweet = true

[foo]
bar = "some string"
integers = [ 1, 2, 3 ]
colors = [ "red", "yellow", "green" ]
nested_arrays_of_ints = [ [ 1, 2 ], [3, 4, 5] ]
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
```

### Reading Examples

```bash
toml sample.toml foo.bar
# Output: some string

toml sample.toml foo.integers
# Output: [ 1, 2, 3 ]

toml sample.toml foo.integers[1]
# Output: 2

toml sample.toml foo.nested_arrays_of_ints[0][0]
# Output: 1

toml sample.toml foo.name.first
# Output: Tom

toml sample.toml products[1].sku
# Output: 284758393
```

### Writing Examples

```bash
toml sample.toml foo.bar some_value
# Writes "some_value" string to foo.bar

toml sample.toml foo.integers [1, 2, 3]
# Writes [1, 2, 3] array to foo.integers

toml sample.toml foo.integers[1] 2
# Writes 2 to the second entry of foo.integers array

toml sample.toml foo.integers[] 5
# Adds 5 to the end of foo.integers array

toml sample.toml foo.integers[] [5, 6, 8]
# Adds three elements (5, 6, 8) to the end of foo.integers array

toml sample.toml products[1].name Nauk
# Sets value "Nauk" to the second element of products array table for key name
```

### Removing Examples

```bash
toml -r sample.toml foo.integers
# Removes integers from foo

toml -r sample.toml foo.integers[:2]
# Removes the last 2 elements from foo.integers array

toml -r sample.toml foo.integers[2:]
# Removes the first 2 elements from foo.integers array

toml -r sample.toml foo.integers[1]
# Removes the second element from foo.integers array

toml -r sample.toml foo.integers[0,2,4]
# Removes elements at indices 0, 2, and 4 from foo.integers array

toml -r sample.toml foo.nested_arrays_of_ints[0][:1]
# Removes the last element from the first nested array in foo.nested_arrays_of_ints
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
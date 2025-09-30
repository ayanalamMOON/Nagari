# Nagari Standard Library

This directory contains the standard library modules for the Nagari programming language.

## Overview

The Nagari standard library provides a comprehensive set of modules for common programming tasks, including:

- **Core functions**: Built-in types and utilities (`core.nag`)
- **String manipulation**: Text processing functions
- **HTTP operations**: Web requests and responses (`http.nag`)
- **File system**: File and directory operations (`fs.nag`)
- **Cryptography**: Encryption and hashing (`crypto.nag`)
- **Mathematics**: Mathematical functions and constants (`math.nag`)
- **Database**: Database connectivity (`db.nag`)
- **JSON processing**: JSON parsing and serialization (`json.nag`)
- **Date/Time**: Date and time utilities (`time.nag`)
- **Operating system**: OS-specific operations (`os.nag`)

## Core Module (`core.nag`)

### Built-in Types and Conversion Functions

```nagari
# Type conversion
len(obj)        # Get length of object
type(obj)       # Get type name as string
str(obj)        # Convert to string
int(obj)        # Convert to integer
float(obj)      # Convert to float
bool(obj)       # Convert to boolean
print(*args)    # Print values to console
```

### String Manipulation Functions

#### Capitalization Functions

```nagari
str_capitalize(s: str) -> str
```
Capitalize the first character of a string, making it uppercase and the rest lowercase.

**Examples:**
```nagari
str_capitalize("hello world")  # "Hello world"
str_capitalize("HELLO")        # "Hello"
str_capitalize("")            # ""
```

```nagari
str_title(s: str) -> str
```
Convert string to title case, capitalizing the first letter of each word.

**Examples:**
```nagari
str_title("hello world")           # "Hello World"
str_title("the quick brown fox")   # "The Quick Brown Fox"
str_title("already Title Case")    # "Already Title Case"
```

#### String Transformation Functions

```nagari
str_reverse(s: str) -> str
```
Reverse a string, returning characters in opposite order.

**Examples:**
```nagari
str_reverse("hello")     # "olleh"
str_reverse("Nagari")    # "iragaN"
str_reverse("racecar")   # "racecar" (palindrome)
```

```nagari
str_count(s: str, substring: str) -> int
```
Count non-overlapping occurrences of substring in string.

**Examples:**
```nagari
str_count("hello world", "l")      # 3
str_count("banana", "ana")         # 1
str_count("hello world", "xyz")    # 0
str_count("", "")                  # 1
```

#### String Padding Functions

```nagari
str_pad_left(s: str, width: int, fillchar: str = " ") -> str
```
Pad string on the left to reach specified width.

**Parameters:**
- `s`: The string to pad
- `width`: Target width of resulting string
- `fillchar`: Character to use for padding (default: space)

**Examples:**
```nagari
str_pad_left("hello", 10)        # "     hello"
str_pad_left("hello", 10, "*")   # "*****hello"
str_pad_left("hello", 8, "0")    # "000hello"
str_pad_left("hello", 3)         # "hello" (no padding needed)
```

```nagari
str_pad_right(s: str, width: int, fillchar: str = " ") -> str
```
Pad string on the right to reach specified width.

**Examples:**
```nagari
str_pad_right("hello", 10)       # "hello     "
str_pad_right("hello", 10, "*")  # "hello*****"
str_pad_right("test", 8, "0")    # "test0000"
```

```nagari
str_center(s: str, width: int, fillchar: str = " ") -> str
```
Center string within specified width by adding padding on both sides.

**Examples:**
```nagari
str_center("hello", 11)          # "   hello   "
str_center("hello", 10)          # "  hello   " (extra padding on right)
str_center("test", 10, "*")      # "***test***"
```

## Usage Examples

### Text Formatting

```nagari
# Format user names
def format_name(first: str, last: str) -> str:
    return str_title(first + " " + last)

print(format_name("john", "doe"))  # "John Doe"
```

### String Processing

```nagari
# Count vowels in text
def count_vowels(text: str) -> int:
    vowels = "aeiouAEIOU"
    total = 0
    for vowel in vowels:
        total += str_count(text.lower(), vowel.lower())
    return total

print(count_vowels("Hello World"))  # 3
```

### Text Alignment

```nagari
# Create a simple table
def print_table(headers: list[str], rows: list[list[str]]):
    # Print headers
    for header in headers:
        print(str_center(header, 15), end="")
    print()

    # Print separator
    print("-" * (15 * len(headers)))

    # Print rows
    for row in rows:
        for cell in row:
            print(str_pad_right(cell, 15), end="")
        print()

print_table(
    ["Name", "Age", "City"],
    [
        ["Alice", "30", "New York"],
        ["Bob", "25", "Los Angeles"],
        ["Charlie", "35", "Chicago"]
    ]
)
```

## Installation and Usage

The standard library is automatically available when using the Nagari compiler. Import specific modules as needed:

```nagari
# Core functions are available by default
print(str_capitalize("hello"))

# Other modules need to be imported
from "http" import { get, post }
from "fs" import { readFile, writeFile }
from "math" import { sin, cos, pi }
```

## Development and Testing

To test the string functions:

```bash
# Run the string function tests
nag run tests/fixtures/string_functions_test.nag

# Or test individual functions in the REPL
nag repl
>>> str_capitalize("hello world")
"Hello world"
>>> str_reverse("nagari")
"iragan"
```

## Contributing

When adding new functions to the standard library:

1. **Add the function to the appropriate `.nag` file** in `stdlib/`
2. **Implement the JavaScript equivalent** in `src/nagari-runtime/src/builtins.ts`
3. **Create comprehensive tests** in `tests/fixtures/`
4. **Update documentation** in `docs/api-reference.md` and this README
5. **Add usage examples** to demonstrate the functionality

### Function Naming Convention

- Use `snake_case` for function names
- Prefix string functions with `str_` for clarity
- Use descriptive names that indicate the function's purpose
- Follow Python naming conventions where applicable

### Error Handling

- Handle edge cases gracefully (empty strings, invalid inputs)
- Provide meaningful error messages for invalid parameters
- Ensure compatibility between Nagari and JavaScript implementations

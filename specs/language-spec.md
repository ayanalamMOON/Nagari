# Nagari Language Specification

## Version 0.1

### 1. Overview

Nagari is a modern, Pythonic scripting language that transpiles to JavaScript. It features:

- Clean, indentation-based syntax inspired by Python
- Optional static typing with runtime flexibility
- Native async/await support
- Full JavaScript ecosystem compatibility
- JSX support for React development
- Transpilation to clean, readable JavaScript (ES6+)

### 2. Design Goals

- **Developer Experience**: Python-like syntax for JavaScript developers
- **Ecosystem Compatibility**: Seamless interop with React, Vue, Express, and npm packages
- **Universal Runtime**: Works in browsers, Node.js, and web frameworks
- **Type Safety**: Optional static typing with TypeScript-like annotations
- **Modern JavaScript**: Compiles to clean ES6+ code

### 2. Lexical Structure

#### 2.1 Character Set

Nagari source files are encoded in UTF-8.

#### 2.2 Comments

Single-line comments begin with `#` and continue to the end of the line.

```nag
# This is a comment
x = 5  # This is also a comment
```

#### 2.3 Indentation

Nagari uses indentation to denote block structure. Indentation must be consistent within a block and must use spaces (4 spaces recommended).

#### 2.4 Keywords

```
def, return, if, elif, else, for, while, match, case,
import, from, async, await, break, continue, true, false, none
```

#### 2.5 Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores.

### 3. Data Types

#### 3.1 Primitive Types

- `int`: 64-bit signed integer
- `float`: 64-bit floating point
- `str`: UTF-8 string
- `bool`: Boolean (`true` or `false`)
- `none`: Null value

#### 3.2 Collection Types

- `list[T]`: Dynamic array
- `dict[K, V]`: Hash map

#### 3.3 Type Annotations

Type annotations are optional but recommended:

```nag
name: str = "Alice"
age: int = 25
scores: list[float] = [95.5, 87.2, 92.0]
```

### 4. Expressions

#### 4.1 Literals

```nag
42          # int
3.14        # float
"hello"     # str
true        # bool
false       # bool
none        # none
```

#### 4.2 Binary Operators

```nag
+, -, *, /, %           # Arithmetic
==, !=, <, >, <=, >=    # Comparison
```

#### 4.3 Function Calls

```nag
print("Hello")
add(5, 3)
```

### 5. Statements

#### 5.1 Assignment

```nag
x = 10
name: str = "Alice"
```

#### 5.2 Function Definition

```nag
def greet(name: str = "world") -> str:
    return "Hello, " + name + "!"

async def fetch_data() -> dict:
    return await http.get("https://api.example.com")
```

#### 5.3 Control Flow

##### If Statements

```nag
if x > 0:
    print("positive")
elif x == 0:
    print("zero")
else:
    print("negative")
```

##### Loops

```nag
for item in items:
    print(item)

while count > 0:
    count = count - 1
```

##### Pattern Matching

```nag
match value:
    case 0:
        print("zero")
    case 1:
        print("one")
    case _:
        print("other")
```

### 6. Modules and Imports

```nag
import math
from http import get, post
```

### 7. Async Programming

```nag
async def main():
    result = await fetch_data()
    print(result)
```

### 8. Error Handling

Error handling is implemented through return values and explicit error checking (to be expanded in future versions).

### 9. Standard Library

The standard library includes modules for:

- `core`: Built-in functions
- `http`: HTTP client/server
- `fs`: File system operations
- `json`: JSON encoding/decoding
- `math`: Mathematical functions
- `time`: Time and date utilities
- `os`: Operating system interface
- `db`: Database connectivity
- `crypto`: Cryptographic functions

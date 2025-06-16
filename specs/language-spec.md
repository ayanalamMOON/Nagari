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
f"Hello {name}"  # f-string/template literal
true        # bool
false       # bool
none        # none
```

#### 4.2 Binary Operators

```nag
+, -, *, /, %, **        # Arithmetic (** for exponentiation)
==, !=, <, >, <=, >=     # Comparison
and, or, not             # Logical
in, not in               # Membership
is, is not               # Identity
```

#### 4.3 Function Calls

```nag
print("Hello")
add(5, 3)
```

#### 4.4 List and Dictionary Comprehensions

```nag
# List comprehension
squares = [x**2 for x in range(10)]
evens = [x for x in numbers if x % 2 == 0]

# Dictionary comprehension
word_lengths = {word: len(word) for word in words}
```

#### 4.5 Lambda Expressions

```nag
square = lambda x: x**2
filter_func = lambda x: x > 0
```

#### 4.6 Generator Expressions

```nag
# Generator expression
squares_gen = (x**2 for x in range(10))
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
#### 5.4 Context Management (With Statements)

```nag
with open("file.txt") as f:
    content = f.read()
    print(content)

with database.connection() as conn:
    result = conn.query("SELECT * FROM users")
```

#### 5.5 Exception Handling

```nag
try:
    result = risky_operation()
except ValueError as e:
    print(f"ValueError: {e}")
except Exception as e:
    print(f"General error: {e}")
finally:
    cleanup()

# Raising exceptions
raise ValueError("Invalid input")
raise CustomError("Something went wrong") from original_error
```

#### 5.6 Generator Functions

```nag
def fibonacci():
    a, b = 0, 1
    while true:
        yield a
        a, b = b, a + b

def process_items(items):
    for item in items:
        processed = transform(item)
        yield processed
```

#### 5.7 Decorators

```nag
@property
def age(self):
    return self._age

@age.setter
def age(self, value):
    if value < 0:
        raise ValueError("Age cannot be negative")
    self._age = value

@async_timeout(5.0)
async def fetch_data():
    return await http.get("https://api.example.com")
```

### 6. Advanced Type System

#### 6.1 Enhanced Type Annotations

```nag
# Union types
def process_id(user_id: int | str) -> User:
    return find_user(user_id)

# Generic types
def get_first[T](items: list[T]) -> T | none:
    return items[0] if items else none

# Callable types
def apply_func(func: (int, int) -> int, a: int, b: int) -> int:
    return func(a, b)

# Tuple types
def get_coordinates() -> (float, float):
    return (10.5, 20.3)
```

#### 6.2 Type Aliases

```nag
type UserId = int | str
type Point = (float, float)
type Handler[T] = (T) -> none

def process_user(user_id: UserId) -> User:
    return find_user(user_id)
```

### 7. Pattern Matching (Enhanced)

```nag
match value:
    case 0:
        print("zero")
    case x if x > 0:
        print(f"positive: {x}")
    case [first, *rest]:
        print(f"list starting with {first}")
    case {"name": name, "age": age} if age >= 18:
        print(f"Adult: {name}")
    case Point(x, y) if x == y:
        print("Point on diagonal")
    case _:
        print("other")
```

### 8. JavaScript Interoperability (Enhanced)

#### 8.1 Direct JavaScript Code

```nag
# Inline JavaScript
result = js("Math.random()")
dom_element = js("document.getElementById('app')")

# JavaScript blocks
complex_operation = js {
    const data = await fetch('/api/data');
    const json = await data.json();
    return json.results;
}
```

#### 8.2 JSX Integration

```nag
# JSX components
def TodoItem(props):
    return (
        <li className={props.completed ? "completed" : ""}>
            <input
                type="checkbox"
                checked={props.completed}
                onChange={props.onToggle}
            />
            <span>{props.text}</span>
            <button onClick={props.onDelete}>Delete</button>
        </li>
    )

# JSX fragments
def MultipleElements():
    return (
        <>
            <h1>Title</h1>
            <p>Content</p>
        </>
    )
```

#### 8.3 Promise and Async Integration

```nag
# Promise methods
async def fetch_all_data():
    results = await Promise.all([
        fetch_user_data(),
        fetch_settings(),
        fetch_notifications()
    ])
    return results

# Promise chaining (if needed)
def legacy_async():
    return fetch_data()
        .then(lambda data: process_data(data))
        .catch(lambda error: handle_error(error))
```

### 9. Module System (Enhanced)

#### 9.1 Import Variations

```nag
# Standard imports
import math
import json
from http import get, post, put
from "react" import React, { useState, useEffect }

# Aliased imports
import numpy as np
from "lodash" import { map as lodash_map }

# Dynamic imports
async def load_module():
    module = await import("./dynamic-module.nag")
    return module.process_data()

# Re-exports
from "./utils" import helper_function
export { helper_function }
```

#### 9.2 Export Variations

```nag
# Named exports
export def calculate(x, y):
    return x + y

export const PI = 3.14159

# Default export
def main_function():
    return "Hello World"

export default main_function

# Export all
export * from "./utilities"
```

### 10. Standard Library (Updated)

The standard library includes comprehensive modules for:

- `core`: Built-in functions (`len`, `type`, `str`, `int`, `float`, `bool`, `print`)
- `math`: Mathematical functions, constants (π, e), trigonometry, statistics
- `fs`: File system operations, path utilities, directory management
- `http`: HTTP client/server, request/response handling, WebSocket support
- `json`: JSON parsing/serialization, schema validation, pretty printing
- `time`: Date/time manipulation, timezone handling, formatting
- `os`: Operating system interface, environment variables, process management
- `db`: Database connectivity, ORM functionality, query builders
- `crypto`: Cryptographic functions, hashing, encryption, digital signatures

#### 10.1 Core Module Usage

```nag
from core import len, type, print

# Built-in functions
length = len([1, 2, 3, 4])  # 4
data_type = type("hello")   # "str"
print("Debug:", length, data_type)
```

#### 10.2 Standard Library Examples

```nag
# Math operations
from math import sqrt, sin, cos, pi
result = sqrt(16)  # 4.0
angle = sin(pi / 2)  # 1.0

# File operations
from fs import read_file, write_file, exists
if exists("config.json"):
    config = json.parse(read_file("config.json"))

# HTTP requests
from http import get, post
response = await get("https://api.example.com/data")
user_data = await post("https://api.example.com/users", {
    "name": "Alice",
    "email": "alice@example.com"
})
```

### 11. Performance and Optimization

#### 11.1 Type Hints for Optimization

```nag
# Type hints help with transpilation optimization
def calculate_distance(x1: float, y1: float, x2: float, y2: float) -> float:
    return math.sqrt((x2 - x1)**2 + (y2 - y1)**2)

# Generic functions with constraints
def sort_items[T: Comparable](items: list[T]) -> list[T]:
    return sorted(items)
```

#### 11.2 Memory Management

```nag
# Context managers for resource cleanup
with database.connection() as conn:
    with conn.transaction():
        conn.execute("INSERT INTO users ...")
        # Automatic cleanup and transaction handling
```

### 12. Future Language Features (Planned)

- **Metaprogramming**: Macro system for compile-time code generation
- **Ownership/Borrowing**: Optional memory safety features
- **Parallel Processing**: Built-in parallelism and concurrency primitives
- **FFI**: Foreign Function Interface for C/C++/Rust integration
- **WASM Target**: WebAssembly compilation target
- **Mobile Support**: React Native and mobile framework integration

# Nagari Language API Reference

This document provides a comprehensive reference for the Nagari programming language, including all built-in functions, syntax, types, and standard library modules.

## Table of Contents

1. [Basic Syntax](#basic-syntax)
2. [Data Types](#data-types)
3. [Control Flow](#control-flow)
4. [Functions](#functions)
5. [Classes](#classes)
6. [Built-in Functions](#built-in-functions)
7. [Standard Library](#standard-library)
8. [JavaScript Interop](#javascript-interop)
9. [React/JSX Support](#reactjsx-support)
10. [Async/Await](#asyncawait)

## Basic Syntax

### Variables and Assignment

```nagari
# Variable assignment
name = "Alice"
age = 30
is_active = true

# Type annotations (optional)
name: str = "Alice"
age: int = 30
score: float = 95.5

# Multiple assignment
x, y = 10, 20
a, b, c = [1, 2, 3]
```

### Comments

```nagari
# This is a single-line comment

"""
This is a multi-line comment
or docstring
"""

def function_with_docstring():
    """
    This function has a docstring
    that explains what it does
    """
    pass
```

### Indentation

Nagari uses indentation to define code blocks (like Python):

```nagari
if condition:
    # This is inside the if block
    do_something()

    if nested_condition:
        # Nested block
        do_something_else()

# Back to the original level
continue_execution()
```

## Data Types

### Primitive Types

```nagari
# Integers
count = 42
negative = -17

# Floating point
pi = 3.14159
scientific = 1.23e-4

# Strings
name = "Alice"
message = 'Hello World'
multiline = """
This is a
multiline string
"""

# Booleans
is_valid = true
is_empty = false

# None/null
value = none
```

### Collection Types

#### Lists

```nagari
# List creation
numbers = [1, 2, 3, 4, 5]
mixed = [1, "hello", true, none]
empty = []

# List operations
numbers.append(6)           # Add to end
numbers.insert(0, 0)        # Insert at index
numbers.remove(3)           # Remove first occurrence
popped = numbers.pop()      # Remove and return last
length = len(numbers)       # Get length

# List comprehension
squares = [x**2 for x in range(10)]
evens = [x for x in numbers if x % 2 == 0]
```

#### Dictionaries

```nagari
# Dictionary creation
person = {"name": "Alice", "age": 30}
empty_dict = {}

# Dictionary operations
person["email"] = "alice@example.com"  # Add/update
age = person["age"]                    # Access
person.pop("age")                      # Remove key
keys = person.keys()                   # Get keys
values = person.values()               # Get values
items = person.items()                 # Get key-value pairs

# Dictionary comprehension
squares = {x: x**2 for x in range(5)}
```

### Type Annotations

```nagari
# Function with type annotations
def calculate_area(length: float, width: float) -> float:
    return length * width

# Variable type annotations
numbers: list[int] = [1, 2, 3]
user: dict[str, any] = {"name": "Alice"}

# Optional types
from typing import Optional
name: Optional[str] = none

# Union types
from typing import Union
id_value: Union[int, str] = "abc123"

# Generic types
def process_list(items: list[T]) -> list[T]:
    return items
```

## Control Flow

### Conditional Statements

```nagari
# If statement
if age >= 18:
    print("Adult")
elif age >= 13:
    print("Teenager")
else:
    print("Child")

# Conditional expression (ternary)
status = "adult" if age >= 18 else "minor"

# Multiple conditions
if age >= 18 and has_license:
    print("Can drive")

if name == "admin" or role == "superuser":
    print("Has admin access")
```

### Loops

#### For Loops

```nagari
# Iterate over sequence
for item in [1, 2, 3, 4, 5]:
    print(item)

# Iterate over string
for char in "hello":
    print(char)

# Iterate over dictionary
for key in person:
    print(f"{key}: {person[key]}")

for key, value in person.items():
    print(f"{key}: {value}")

# Range-based loops
for i in range(10):           # 0 to 9
    print(i)

for i in range(1, 11):        # 1 to 10
    print(i)

for i in range(0, 10, 2):     # 0, 2, 4, 6, 8
    print(i)

# Enumerate
for index, value in enumerate(["a", "b", "c"]):
    print(f"{index}: {value}")
```

#### While Loops

```nagari
# Basic while loop
count = 0
while count < 5:
    print(count)
    count += 1

# While with else
while condition:
    do_something()
else:
    # Executes if loop completes normally
    print("Loop finished")
```

#### Loop Control

```nagari
for i in range(10):
    if i == 3:
        continue  # Skip rest of iteration
    if i == 7:
        break     # Exit loop
    print(i)
```

### Pattern Matching

```nagari
match value:
    case 1:
        print("One")
    case 2 | 3:
        print("Two or three")
    case x if x > 10:
        print(f"Large number: {x}")
    case _:
        print("Something else")

# Match with data structures
match data:
    case {"type": "user", "name": name}:
        print(f"User: {name}")
    case {"type": "admin", "permissions": perms}:
        print(f"Admin with permissions: {perms}")
    case _:
        print("Unknown data type")
```

## Functions

### Function Definition

```nagari
# Basic function
def greet(name):
    return f"Hello, {name}!"

# Function with type annotations
def add(a: int, b: int) -> int:
    return a + b

# Function with default parameters
def greet(name: str = "World") -> str:
    return f"Hello, {name}!"

# Function with variable arguments
def sum_all(*args):
    return sum(args)

# Function with keyword arguments
def create_user(**kwargs):
    return {"name": kwargs.get("name"), "age": kwargs.get("age")}
```

### Lambda Functions

```nagari
# Lambda expressions
square = lambda x: x ** 2
add = lambda a, b: a + b

# Used with higher-order functions
numbers = [1, 2, 3, 4, 5]
squares = map(lambda x: x**2, numbers)
evens = filter(lambda x: x % 2 == 0, numbers)
```

### Decorators

```nagari
# Function decorator
def timing_decorator(func):
    def wrapper(*args, **kwargs):
        start = Date.now()
        result = func(*args, **kwargs)
        end = Date.now()
        print(f"{func.name} took {end - start}ms")
        return result
    return wrapper

@timing_decorator
def slow_function():
    # Some slow operation
    pass
```

## Classes

### Class Definition

```nagari
class Person:
    # Class variable
    species = "Homo sapiens"

    def __init__(self, name: str, age: int):
        self.name = name
        self.age = age
        self._private_var = "secret"

    def greet(self) -> str:
        return f"Hello, I'm {self.name}"

    def get_age(self) -> int:
        return self.age

    @staticmethod
    def is_adult(age: int) -> bool:
        return age >= 18

    @classmethod
    def from_birth_year(cls, name: str, birth_year: int):
        age = 2025 - birth_year
        return cls(name, age)

# Usage
person = Person("Alice", 30)
print(person.greet())
print(Person.is_adult(25))
```

### Inheritance

```nagari
class Animal:
    def __init__(self, name: str):
        self.name = name

    def speak(self) -> str:
        return "Some sound"

class Dog(Animal):
    def __init__(self, name: str, breed: str):
        super().__init__(name)
        self.breed = breed

    def speak(self) -> str:
        return "Woof!"

    def fetch(self) -> str:
        return f"{self.name} is fetching!"

# Usage
dog = Dog("Buddy", "Golden Retriever")
print(dog.speak())
print(dog.fetch())
```

## Built-in Functions

### Type Conversion

```nagari
# String conversion
str(123)        # "123"
str(true)       # "true"

# Numeric conversion
int("123")      # 123
float("3.14")   # 3.14
bool(1)         # true
bool(0)         # false

# Collection conversion
list("hello")   # ["h", "e", "l", "l", "o"]
dict([["a", 1], ["b", 2]])  # {"a": 1, "b": 2}
```

### Math Functions

```nagari
abs(-5)         # 5
max(1, 2, 3)    # 3
min(1, 2, 3)    # 1
round(3.14159, 2)  # 3.14
sum([1, 2, 3])  # 6

# Math module
from Math import { sin, cos, pi, sqrt }
print(sin(pi / 2))  # 1.0
print(sqrt(16))     # 4.0
```

### Sequence Functions

```nagari
len([1, 2, 3])      # 3
range(5)            # [0, 1, 2, 3, 4]
enumerate(["a", "b"])  # [(0, "a"), (1, "b")]
zip([1, 2], ["a", "b"])  # [(1, "a"), (2, "b")]

# Higher-order functions
map(lambda x: x*2, [1, 2, 3])     # [2, 4, 6]
filter(lambda x: x>2, [1, 2, 3, 4])  # [3, 4]
any([false, true, false])          # true
all([true, true, false])           # false
```

### Input/Output

```nagari
# Console output
print("Hello, World!")
print("Name:", name, "Age:", age)

# Console input (browser/Node.js)
name = input("Enter your name: ")

# Error output
import sys
sys.stderr.write("Error message\n")
```

## Standard Library

### Core Module

```nagari
from "core" import { isinstance, hasattr, getattr, setattr }

isinstance(obj, str)     # Check if obj is string
hasattr(obj, "method")   # Check if obj has method
getattr(obj, "prop")     # Get property value
setattr(obj, "prop", value)  # Set property value
```

### HTTP Module

```nagari
from "http" import { get, post, put, delete }

# HTTP requests
async def fetch_data():
    response = await get("https://api.example.com/users")
    return response.json()

async def create_user(user_data):
    response = await post("https://api.example.com/users", {
        "headers": {"Content-Type": "application/json"},
        "body": JSON.stringify(user_data)
    })
    return response.json()
```

### File System Module

```nagari
from "fs" import { readFile, writeFile, mkdir, exists }

async def process_file(filename):
    if await exists(filename):
        content = await readFile(filename, "utf8")
        processed = content.upper()
        await writeFile(f"processed_{filename}", processed)
```

### JSON Module

```nagari
from "json" import { parse, stringify }

# JSON operations
data = {"name": "Alice", "age": 30}
json_string = stringify(data)
parsed_data = parse(json_string)
```

### Math Module

```nagari
from "math" import {
    sin, cos, tan, asin, acos, atan,
    sqrt, pow, log, exp,
    pi, e,
    floor, ceil, round
}

# Mathematical operations
area = pi * pow(radius, 2)
hypotenuse = sqrt(pow(a, 2) + pow(b, 2))
```

### Time Module

```nagari
from "time" import { now, sleep, format_date }

# Time operations
current_time = now()
formatted = format_date(current_time, "YYYY-MM-DD")

async def delayed_operation():
    await sleep(1000)  # Sleep for 1 second
    print("Operation completed")
```

## JavaScript Interop

### Importing JavaScript Modules

```nagari
# Import from npm packages
from "lodash" import { map, filter, reduce }
from "moment" import moment
from "axios" import axios

# Import Node.js modules
from "fs" import { readFileSync }
from "path" import { join, dirname }

# Import browser APIs
from "DOM" import { getElementById, querySelector }
```

### Using JavaScript Functions

```nagari
# JavaScript functions work seamlessly
from "lodash" import { chunk, flatten }

data = [1, 2, 3, 4, 5, 6]
chunks = chunk(data, 2)  # [[1, 2], [3, 4], [5, 6]]
flat = flatten(chunks)   # [1, 2, 3, 4, 5, 6]
```

### Direct JavaScript Code

```nagari
# Embed JavaScript directly when needed
result = js("""
    function complexOperation(data) {
        // Complex JavaScript logic
        return processedData;
    }
    complexOperation(data);
""")
```

## React/JSX Support

### React Components

```nagari
from "react" import React, { useState, useEffect }

def Counter(props):
    count, setCount = useState(0)

    useEffect(() => {
        document.title = f"Count: {count}"
    }, [count])

    def handleClick():
        setCount(count + 1)

    return <div className={props.className}>
        <h1>Count: {count}</h1>
        <button onClick={handleClick}>
            Increment
        </button>
    </div>

export default Counter
```

### JSX Syntax

```nagari
# JSX elements
element = <div>Hello, World!</div>

# JSX with attributes
button = <button onClick={handleClick} disabled={isLoading}>
    Click me
</button>

# JSX with children
card = <div className="card">
    <h2>Title</h2>
    <p>Content goes here</p>
</div>

# JSX fragments
fragment = <>
    <h1>Title</h1>
    <p>Paragraph</p>
</>

# Conditional rendering
content = <div>
    {isLoggedIn ? <Dashboard /> : <LoginForm />}
</div>

# List rendering
items = <ul>
    {todos.map(todo =>
        <li key={todo.id}>{todo.text}</li>
    )}
</ul>
```

### React Hooks

```nagari
from "react" import {
    useState, useEffect, useCallback, useMemo,
    useRef, useContext, useReducer
}

def MyComponent():
    # State hook
    count, setCount = useState(0)

    # Effect hook
    useEffect(() => {
        print("Component mounted or count changed")
        return () => print("Cleanup")
    }, [count])

    # Callback hook
    handleClick = useCallback(() => {
        setCount(count + 1)
    }, [count])

    # Memo hook
    expensiveValue = useMemo(() => {
        return expensiveCalculation(count)
    }, [count])

    # Ref hook
    inputRef = useRef(none)

    return <div>
        <input ref={inputRef} />
        <button onClick={handleClick}>
            Count: {count}
        </button>
    </div>
```

## Async/Await

### Async Functions

```nagari
async def fetch_user_data(user_id):
    try:
        response = await fetch(f"/api/users/{user_id}")
        if not response.ok:
            raise Exception(f"HTTP {response.status}")
        return await response.json()
    except Exception as e:
        print(f"Error fetching user: {e}")
        return none
```

### Promise Handling

```nagari
from "Promise" import { all, race, resolve, reject }

async def parallel_requests():
    # Wait for all promises
    results = await all([
        fetch("/api/users"),
        fetch("/api/posts"),
        fetch("/api/comments")
    ])

    return {
        "users": await results[0].json(),
        "posts": await results[1].json(),
        "comments": await results[2].json()
    }

async def race_requests():
    # Wait for first to complete
    fastest = await race([
        fetch("/api/fast-endpoint"),
        fetch("/api/slow-endpoint")
    ])

    return await fastest.json()
```

### Error Handling with Async

```nagari
async def robust_operation():
    try:
        result = await risky_operation()
        return result
    except NetworkError as e:
        print(f"Network error: {e}")
        return none
    except TimeoutError as e:
        print(f"Timeout: {e}")
        return none
    except Exception as e:
        print(f"Unexpected error: {e}")
        raise
    finally:
        print("Cleanup completed")
```

## Error Handling

### Try/Catch/Finally

```nagari
try:
    result = risky_operation()
    print(f"Success: {result}")
except SpecificError as e:
    print(f"Specific error: {e}")
except (TypeError, ValueError) as e:
    print(f"Type or value error: {e}")
except Exception as e:
    print(f"General error: {e}")
    raise  # Re-raise the exception
else:
    print("No exceptions occurred")
finally:
    print("This always executes")
```

### Custom Exceptions

```nagari
class CustomError(Exception):
    def __init__(self, message, code=None):
        super().__init__(message)
        self.code = code

def validate_input(value):
    if value < 0:
        raise CustomError("Value must be positive", code="NEGATIVE_VALUE")
    if value > 100:
        raise CustomError("Value too large", code="VALUE_TOO_LARGE")
    return value
```

This API reference covers the core features of the Nagari language. For more advanced topics and framework-specific integrations, see the specialized documentation files.

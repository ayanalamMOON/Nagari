# Getting Started with Nagari

## Installation

### Prerequisites

- Rust 1.70 or later
- Git
- Node.js 14+ or Bun 1.0+ (for running transpiled code)

### Recommended: Install Bun for Better Performance

Nagari works with both Node.js and Bun, but **Bun provides 4x faster performance**:

```bash
# macOS/Linux
curl -fsSL https://bun.sh/install | bash

# Windows (PowerShell)
powershell -c "irm bun.sh/install.ps1 | iex"

# Verify
bun --version
```

The Nagari CLI automatically detects and uses Bun if available, falling back to Node.js otherwise.

### Building from Source

1. Clone the repository:

```bash
git clone https://github.com/nagari-lang/nagari.git
cd nagari
```

2. Build the compiler:

```bash
cd nagari-compiler
cargo build --release
```

3. Build the virtual machine:

```bash
cd ../nagari-vm
cargo build --release
```

4. Add the binaries to your PATH:

```bash
# On Unix-like systems
export PATH=$PATH:/path/to/nagari/nagari-compiler/target/release:/path/to/nagari/nagari-vm/target/release

# On Windows
set PATH=%PATH%;C:\path\to\nagari\nagari-compiler\target\release;C:\path\to\nagari\nagari-vm\target\release
```

## Your First Nagari Program

Create a file called `hello.nag`:

```nag
def greet(name: str = "world") -> str:
    return "Hello, " + name + "!"

print(greet())
print(greet("Nagari"))
```

Compile and run it:

```bash
# Compile to bytecode
nagc hello.nag

# Run the bytecode
nagrun hello.nac
```

Output:

```
Hello, world!
Hello, Nagari!
```

## Basic Syntax

### Variables and Types

```nag
# Type annotations are optional but recommended
name: str = "Alice"
age: int = 25
height: float = 5.6
is_student: bool = true
scores: list[float] = [95.5, 87.2, 92.0]
```

### Functions

```nag
def add(a: int, b: int) -> int:
    return a + b

def greet(name: str = "friend") -> str:
    return "Hello, " + name

# Async functions
async def fetch_data(url: str) -> dict:
    return await http.get(url)
```

### Control Flow

```nag
# If statements
if age >= 18:
    print("Adult")
elif age >= 13:
    print("Teenager")
else:
    print("Child")

# Loops
for i in range(5):
    print(i)

while count > 0:
    count = count - 1

# Pattern matching
match status:
    case "ok":
        print("Success")
    case "error":
        print("Failed")
    case _:
        print("Unknown")
```

### Collections

```nag
# Lists
numbers: list[int] = [1, 2, 3, 4, 5]
numbers.append(6)
print(numbers[0])  # First element

# Dictionaries
person: dict[str, any] = {
    "name": "Alice",
    "age": 25,
    "skills": ["Python", "Nagari"]
}
print(person["name"])
```

## Standard Library

Import modules from the standard library:

```nag
import math
import http
import fs
import json
from time import sleep
```

### Common Operations

```nag
# Math
result = math.sqrt(16)
area = math.PI * radius * radius

# HTTP requests
async def main():
    response = await http.get("https://api.example.com/data")
    print(response)

# File operations
content = fs.read_file("data.txt")
fs.write_file("output.txt", "Hello, file!")

# JSON
data = {"name": "Alice", "age": 25}
json_str = json.dumps(data)
parsed = json.loads(json_str)
```

## Next Steps

- Read the [Language Specification](specs/language-spec.md)
- Explore the [examples](examples/) directory
- Check out the [standard library documentation](stdlib/)
- Learn about [async programming](docs/async-guide.md)

## Getting Help

- Visit our [GitHub repository](https://github.com/nagari-lang/nagari)
- Read the [FAQ](docs/faq.md)
- Join our [community discussions](https://github.com/nagari-lang/nagari/discussions)

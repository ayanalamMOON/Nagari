# Nagari Frequently Asked Questions (FAQ)

Common questions and answers about the Nagari programming language.

## Table of Contents

1. [General Questions](#general-questions)
2. [Installation and Setup](#installation-and-setup)
3. [Language Features](#language-features)
4. [JavaScript Interoperability](#javascript-interoperability)
5. [Performance](#performance)
6. [Development and Deployment](#development-and-deployment)
7. [Comparison with Other Languages](#comparison-with-other-languages)
8. [Community and Support](#community-and-support)

## General Questions

### What is Nagari?

Nagari is a modern programming language with Python-like syntax that transpiles to JavaScript. It's designed to give developers the expressiveness and readability of Python while leveraging the vast JavaScript ecosystem and runtime capabilities.

### Why would I use Nagari instead of Python or JavaScript?

**Compared to Python:**

- Run anywhere JavaScript runs (browsers, Node.js, edge computing)
- Access to the entire npm ecosystem
- Better async/await integration
- Native React/JSX support

**Compared to JavaScript:**

- Cleaner, more readable syntax
- Optional static typing
- No semicolons or confusing `this` binding
- Python-like control structures and comprehensions

### Is Nagari production-ready?

Nagari is currently in active development. While the core language features are stable, we recommend using it for:

- Prototyping and experimentation
- Educational projects
- Small to medium applications
- Contributing to the language development

For large production systems, consider waiting for the 1.0 release.

### What license is Nagari released under?

Nagari is released under the MIT License, making it free for both personal and commercial use.

## Installation and Setup

### What are the system requirements for Nagari?

**Minimum requirements:**

- Rust 1.70 or later (for compiler development)
- Node.js 16 or later (for runtime)
- 4GB RAM (recommended 8GB)
- 2GB disk space

**Supported platforms:**

- Windows 10/11
- macOS 10.15 or later
- Linux (Ubuntu 20.04+, other distributions)

### Can I install Nagari without installing Rust?

Currently, building Nagari from source requires Rust. We're working on:

- Pre-built binaries for major platforms
- npm package for easy installation
- Docker containers for isolated development

### How do I set up my development environment?

1. **Install dependencies:**

   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install Node.js (via nvm recommended)
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
   nvm install 18
   ```

2. **Clone and build Nagari:**

   ```bash
   git clone https://github.com/nagari-lang/nagari.git
   cd nagari
   ./tools/build.sh
   ```

3. **Add to PATH:**

   ```bash
   export PATH=$PATH:$(pwd)/nagari-compiler/target/release
   ```

### Do I need to know JavaScript to use Nagari?

No, but basic JavaScript knowledge helps with:

- Understanding the runtime environment
- Using JavaScript libraries effectively
- Debugging transpiled code
- Advanced interop scenarios

## Language Features

### Does Nagari support all Python features?

Nagari implements a subset of Python's features with some extensions:

**Supported:**

- Classes and inheritance
- Functions with default parameters
- List/dict comprehensions
- Context managers (`with` statements)
- Decorators
- Async/await
- Type hints

**Not supported (yet):**

- Metaclasses
- Multiple inheritance (complex cases)
- Generator expressions with `yield`
- Some advanced decorators

**Extensions:**

- JSX syntax for React
- JavaScript interop features
- Import syntax for npm modules

### Can I use Python libraries with Nagari?

No, Nagari runs on JavaScript runtime, not Python. However:

- Many Python standard library functions are implemented in Nagari's stdlib
- JavaScript equivalents exist for most Python libraries
- The interop system makes JavaScript libraries feel Python-like

### How does Nagari handle Python's dynamic typing?

Nagari supports both dynamic and static typing:

```nagari
# Dynamic (like Python)
variable = "string"
variable = 42  # OK

# Static (optional)
name: str = "Alice"
age: int = 30

# Runtime type checking available
def typed_function(value: str) -> int:
    if not isinstance(value, str):
        raise TypeError("Expected string")
    return len(value)
```

### Does Nagari support pattern matching?

Yes, Nagari supports pattern matching similar to Python 3.10:

```nagari
match value:
    case int() if value > 0:
        return "positive integer"
    case str() if len(value) > 0:
        return "non-empty string"
    case [first, *rest]:
        return f"list starting with {first}"
    case _:
        return "something else"
```

### How does error handling work?

Nagari uses Python-style exception handling:

```nagari
try:
    risky_operation()
except SpecificError as e:
    handle_specific_error(e)
except Exception as e:
    handle_any_error(e)
finally:
    cleanup()
```

JavaScript errors are automatically converted to Nagari exceptions.

## JavaScript Interoperability

### How do I use JavaScript libraries in Nagari?

Import them like Python modules:

```nagari
# npm packages
import express from "express"
import { useState } from "react"
import axios from "axios"

# Node.js built-ins
import fs from "fs"
import path from "path"

# Use normally
app = express()
response = await axios.get("https://api.example.com")
```

### How do I pass data between Nagari and JavaScript?

The interop system handles conversions automatically:

```nagari
# Nagari to JavaScript
nagari_list = [1, 2, 3]
js_function(nagari_list)  # Automatically converted

# JavaScript to Nagari
js_result = js_function()
nagari_result = nagari.from_js(js_result)  # Explicit conversion if needed

# Complex objects
user_data = {
    "name": "Alice",
    "scores": [95, 87, 92]
}
save_user(user_data)  # Works seamlessly
```

### Can I write React components in Nagari?

Yes! Nagari has first-class JSX support:

```nagari
import React, { useState } from "react"

def TodoApp():
    todos, set_todos = useState([])

    def add_todo(text):
        new_todo = {"id": len(todos), "text": text, "done": false}
        set_todos([*todos, new_todo])

    return (
        <div className="todo-app">
            <h1>My Todos</h1>
            {todos.map(todo => (
                <div key={todo.id}>
                    {todo.text}
                </div>
            ))}
        </div>
    )

export default TodoApp
```

### How do I handle JavaScript's `this` keyword?

Nagari's class methods handle `this` automatically:

```nagari
class MyClass:
    def __init__(self):
        self.value = 42

    def get_value(self):
        return self.value  # 'this' is handled automatically

# Arrow functions in callbacks
button.addEventListener("click", () => {
    self.handle_click()  # Preserves context
})
```

### Can I use TypeScript declaration files?

Yes, Nagari can consume TypeScript declarations for better IDE support:

```bash
# Generate declarations
nagc --declarations src/ --output dist/

# Use with TypeScript projects
import { NagariFunction } from "./dist/module.d.ts"
```

## Performance

### How fast is Nagari compared to Python?

Performance depends on the specific use case:

**Faster than Python for:**

- I/O operations (leverages JavaScript's event loop)
- Web applications (runs natively in browsers)
- String manipulation (JavaScript's optimized string handling)

**Similar to Python for:**

- Basic arithmetic
- Data structure operations
- Most general programming tasks

**Potentially slower for:**

- CPU-intensive numerical computation
- Some advanced algorithms

### How can I optimize Nagari code performance?

1. **Use JavaScript-optimized patterns:**

   ```nagari
   # Use array methods instead of loops
   result = items.filter(lambda x: x > 10).map(lambda x: x * 2)

   # Instead of
   result = []
   for item in items:
       if item > 10:
           result.append(item * 2)
   ```

2. **Leverage async operations:**

   ```nagari
   # Concurrent requests
   tasks = [fetch_data(url) for url in urls]
   results = await asyncio.gather(*tasks)
   ```

3. **Use appropriate data structures:**

   ```nagari
   # Use Set for membership testing
   valid_ids = {1, 2, 3, 4, 5}
   if user_id in valid_ids:  # O(1) instead of O(n)
       process_user()
   ```

### Does Nagari support JIT compilation?

Nagari leverages JavaScript's JIT compilation in V8 and other engines. The transpiled JavaScript code benefits from:

- Runtime optimizations
- Inline caching
- Dead code elimination
- Loop optimization

### Can I profile Nagari applications?

Yes, use JavaScript profiling tools:

```bash
# Node.js profiling
node --prof dist/app.js

# Chrome DevTools for browser applications
# Use --sourcemap flag for better debugging
nagc --sourcemap src/ --output dist/
```

## Development and Deployment

### What IDEs support Nagari?

**VS Code:** Best support with official extension

**Other editors:**

- Vim/Neovim: Syntax highlighting available
- Emacs: Basic mode available
- Sublime Text: Syntax highlighting
- IntelliJ: Community plugin in development

### How do I deploy Nagari applications?

**Web applications:**

```bash
# Build for browser
nagc --target browser src/ --output dist/
# Deploy dist/ folder to web server
```

**Node.js applications:**

```bash
# Build for Node.js
nagc --target node src/ --output dist/
# Deploy and run
node dist/main.js
```

**Serverless functions:**

```bash
# AWS Lambda, Vercel, Netlify Functions
nagc --target serverless src/ --output dist/
```

### Can I use Nagari with existing JavaScript projects?

Yes! Nagari modules can be imported in JavaScript:

```javascript
// Import compiled Nagari module
import { processData } from './nagari-module.js';

// Use Nagari functions in JavaScript
const result = processData(inputData);
```

### How do I handle package management?

Use npm for JavaScript dependencies:

```json
{
  "dependencies": {
    "express": "^4.18.0",
    "react": "^18.2.0",
    "axios": "^1.3.0"
  },
  "devDependencies": {
    "@types/node": "^18.0.0"
  }
}
```

Nagari standard library modules are included with the compiler.

### What testing frameworks can I use?

Standard JavaScript testing frameworks work:

```nagari
# Jest
import { test, expect } from "@jest/globals"

def test_addition():
    expect(add(2, 3)).toBe(5)

# Mocha
import { describe, it } from "mocha"
import { expect } from "chai"

describe("Math functions", () => {
    it("should add numbers correctly", () => {
        expect(add(2, 3)).to.equal(5)
    })
})
```

## Comparison with Other Languages

### How does Nagari compare to TypeScript?

**Similarities:**

- Transpiles to JavaScript
- Optional static typing
- Good tooling support

**Differences:**

- Nagari: Python-like syntax, more readable
- TypeScript: JavaScript superset, larger ecosystem
- Nagari: Built-in async patterns
- TypeScript: More mature, better IDE support

### How does Nagari compare to CoffeeScript?

**Similarities:**

- Alternative syntax for JavaScript
- Compiles to readable JavaScript

**Differences:**

- Nagari: Python-like syntax vs. Ruby-like
- Nagari: Modern async/await support
- Nagari: Active development vs. maintenance mode
- Nagari: Static typing support

### How does Nagari compare to Dart?

**Similarities:**

- Modern language design
- Optional static typing
- Compiles to JavaScript

**Differences:**

- Nagari: Python-like syntax vs. C-like
- Dart: Flutter for mobile vs. web focus
- Nagari: Smaller runtime overhead
- Dart: More mature ecosystem

### Why not just use Python with Pyodide?

**Pyodide advantages:**

- True Python compatibility
- Access to Python scientific libraries

**Nagari advantages:**

- Smaller bundle size
- Better JavaScript integration
- Native performance (no interpretation layer)
- Easier deployment

## Community and Support

### Where can I get help?

1. **Documentation:** Official docs at [nagari-lang.org](https://nagari-lang.org)
2. **GitHub Issues:** Bug reports and feature requests
3. **Discussions:** Community forum on GitHub
4. **Discord:** Real-time chat with developers
5. **Stack Overflow:** Tag questions with `nagari-lang`

### How can I contribute to Nagari?

**Code contributions:**

1. Check the issues page for "good first issue" labels
2. Fork the repository
3. Make changes and add tests
4. Submit a pull request

**Non-code contributions:**

- Documentation improvements
- Bug reports and testing
- Feature suggestions
- Community support

### What's the development roadmap?

**Near term (6 months):**

- Standard library completion
- Better error messages
- IDE improvements
- Package manager

**Medium term (1 year):**

- Language server protocol
- Debugger support
- Performance optimizations
- 1.0 release

**Long term:**

- Native compilation options
- Advanced type system
- Ecosystem growth

### Is there commercial support available?

Currently, Nagari is a community-driven project. Commercial support options are being explored for the future.

### How stable is the language syntax?

The core syntax is relatively stable, but expect changes before 1.0:

- Breaking changes will be announced
- Migration guides will be provided
- Version compatibility will be maintained within major versions

For the latest updates, follow the [official blog](https://nagari-lang.org/blog) and [release notes](https://github.com/nagari-lang/nagari/releases).

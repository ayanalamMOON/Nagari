# REPL Guide

Interactive development with the Nagari Read-Eval-Print Loop.

## Overview

The Nagari REPL provides an interactive environment for exploring the language, testing code snippets, and rapid prototyping. It's perfect for learning Nagari, debugging, and experimenting with new ideas.

## Getting Started

### Starting the REPL

```bash
# Basic REPL
nagari repl

# REPL with specific runtime
nagari repl --runtime node

# REPL with debug mode
nagari repl --debug
```

### First Steps

```
Welcome to Nagari REPL v0.2.0
Type .help for available commands

>>> let message = "Hello, Nagari!"
"Hello, Nagari!"

>>> console.log(message)
Hello, Nagari!
undefined

>>> 2 + 3 * 4
14
```

## Basic Usage

### Variable Declaration

```
>>> let x = 42
42

>>> const pi = 3.14159
3.14159

>>> var name = "Nagari"
"Nagari"
```

### Function Definition

```
>>> function greet(name) {
...   return `Hello, ${name}!`
... }
undefined

>>> greet("World")
"Hello, World!"

>>> let square = (x) => x * x
[Function: square]

>>> square(5)
25
```

### Object and Array Operations

```
>>> let obj = { name: "Nagari", version: "0.2.0" }
{ name: "Nagari", version: "0.2.0" }

>>> obj.name
"Nagari"

>>> let arr = [1, 2, 3, 4, 5]
[1, 2, 3, 4, 5]

>>> arr.map(x => x * 2)
[2, 4, 6, 8, 10]
```

## REPL Commands

The REPL supports several special commands starting with a dot (`.`):

### `.help` - Show Help

```
>>> .help
Available commands:
  .help          Show this help message
  .exit          Exit the REPL
  .clear         Clear the current context
  .load <file>   Load and execute a file
  .save <file>   Save session history to file
  .editor        Enter editor mode
  .break         Exit from multiline input
  .history       Show command history
  .reset         Reset the REPL context
```

### `.load` - Load Files

```
>>> .load examples/math_demo.nag
Loaded: examples/math_demo.nag

>>> factorial(5)
120
```

### `.save` - Save Session

```
>>> .save my-session.nag
Session saved to my-session.nag
```

### `.editor` - Multi-line Editor

```
>>> .editor
// Entering editor mode (^D to finish, ^C to cancel)
function complexFunction(a, b, c) {
  let result = a * b + c;
  if (result > 100) {
    return result / 2;
  }
  return result;
}

>>> complexFunction(10, 15, 5)
75
```

### `.clear` - Clear Context

```
>>> let temp = "temporary"
"temporary"

>>> .clear
Context cleared

>>> temp
ReferenceError: temp is not defined
```

## Advanced Features

### HTTP Requests

```
>>> import { fetch } from 'http'
undefined

>>> let response = await fetch('https://api.github.com/users/octocat')
undefined

>>> let data = await response.json()
undefined

>>> data.name
"The Octocat"
```

### Async/Await Support

```
>>> async function delay(ms) {
...   return new Promise(resolve => setTimeout(resolve, ms))
... }
undefined

>>> await delay(1000)
undefined

>>> console.log("1 second later!")
1 second later!
```

### Module System

```
>>> import { readFile } from 'fs'
undefined

>>> let content = await readFile('package.json', 'utf8')
undefined

>>> JSON.parse(content).name
"my-project"
```

## Configuration

### Environment Setup

```bash
# Set REPL history file
export NAGARI_REPL_HISTORY=~/.nagari_history

# Set default runtime
export NAGARI_DEFAULT_RUNTIME=node

# Enable debug mode
export NAGARI_DEBUG=true
```

### Custom Startup Script

Create a `.nagari_repl.nag` file in your home directory:

```nagari
// Auto-loaded on REPL startup
import { log } from 'console'
import { performance } from 'perf_hooks'

// Helper functions
function time(fn) {
  let start = performance.now()
  let result = fn()
  let end = performance.now()
  log(`Execution time: ${end - start}ms`)
  return result
}

function inspect(obj) {
  return JSON.stringify(obj, null, 2)
}

// Welcome message
log("Custom REPL environment loaded!")
log("Available helpers: time(), inspect()")
```

## Runtime Environments

### Node.js Runtime

```bash
nagari repl --runtime node
```

Features:
- Full Node.js API access
- File system operations
- Network requests
- Process management

```
>>> import { readFileSync } from 'fs'
>>> import { spawn } from 'child_process'
>>> process.version
"v18.17.0"
```

### Browser Runtime

```bash
nagari repl --runtime browser
```

Features:
- DOM manipulation (when available)
- Fetch API
- Web APIs
- Local storage

```
>>> fetch('https://api.example.com/data')
>>> localStorage.setItem('key', 'value')
>>> document.querySelector('#app') // if DOM available
```

### Deno Runtime

```bash
nagari repl --runtime deno
```

Features:
- Secure by default
- Built-in TypeScript
- Standard library
- Web-compatible APIs

## Debugging

### Debug Mode

```bash
nagari repl --debug
```

Provides:
- Detailed error messages
- Execution tracing
- Performance metrics
- Memory usage info

### Error Handling

```
>>> function buggyFunction() {
...   throw new Error("Something went wrong!")
... }
undefined

>>> try {
...   buggyFunction()
... } catch (e) {
...   console.log("Caught:", e.message)
... }
Caught: Something went wrong!
```

### Stack Traces

```
>>> function a() { b() }
>>> function b() { c() }
>>> function c() { throw new Error("Deep error") }

>>> a()
Error: Deep error
    at c (REPL:1:35)
    at b (REPL:1:18)
    at a (REPL:1:18)
    at REPL:1:1
```

## Tips and Tricks

### Quick Calculations

```
>>> // Currency conversion
>>> let usd = 100
>>> let rate = 1.18
>>> usd * rate + " EUR"
"118 EUR"
```

### Data Exploration

```
>>> let data = [
...   { name: "Alice", age: 30 },
...   { name: "Bob", age: 25 },
...   { name: "Charlie", age: 35 }
... ]

>>> data.filter(p => p.age > 28)
[{ name: "Alice", age: 30 }, { name: "Charlie", age: 35 }]

>>> data.map(p => p.name).join(", ")
"Alice, Bob, Charlie"
```

### Testing Code Snippets

```
>>> // Test regular expressions
>>> let email = "user@example.com"
>>> /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
true

>>> // Test date formatting
>>> new Date().toISOString().split('T')[0]
"2024-01-15"
```

### Package Prototyping

```
>>> // Test a function before adding to module
>>> function fibonacci(n) {
...   if (n <= 1) return n
...   return fibonacci(n-1) + fibonacci(n-2)
... }

>>> [0,1,2,3,4,5].map(fibonacci)
[0, 1, 1, 2, 3, 5]

>>> // Satisfied? Save to file
>>> .save fibonacci.nag
```

## Common Patterns

### API Testing

```
>>> async function testAPI(url) {
...   try {
...     let res = await fetch(url)
...     let data = await res.json()
...     console.log("Status:", res.status)
...     console.log("Data:", data)
...     return data
...   } catch (e) {
...     console.error("Error:", e.message)
...   }
... }

>>> testAPI("https://jsonplaceholder.typicode.com/posts/1")
```

### Data Transformation

```
>>> let csv = "name,age,city\nAlice,30,NYC\nBob,25,LA"
>>> let lines = csv.split('\n')
>>> let headers = lines[0].split(',')
>>> let rows = lines.slice(1).map(line => {
...   let values = line.split(',')
...   return headers.reduce((obj, header, i) => {
...     obj[header] = values[i]
...     return obj
...   }, {})
... })
>>> rows
[
  { name: "Alice", age: "30", city: "NYC" },
  { name: "Bob", age: "25", city: "LA" }
]
```

## Troubleshooting

### Common Issues

**REPL won't start:**
```bash
# Check Nagari installation
nagari --version

# Try with specific runtime
nagari repl --runtime node
```

**Module import errors:**
```
>>> // Use relative paths for local modules
>>> import { helper } from './utils.nag'

>>> // Check if file exists
>>> .load utils.nag
```

**Memory issues with large data:**
```
>>> // Clear variables when done
>>> largeArray = null

>>> // Or reset entire context
>>> .clear
```

### Performance Tips

- Use `.clear` to free memory between experiments
- Avoid creating large objects in global scope
- Use `time()` helper to measure execution speed
- Consider `.break` to exit infinite loops

## Integration with Development

### VS Code Integration

The REPL works seamlessly with VS Code:
1. Open integrated terminal
2. Run `nagari repl`
3. Copy/paste code from editor
4. Use `.load` to execute files

### Testing Workflow

```bash
# Edit code in editor
# Test in REPL
nagari repl
>>> .load my-function.nag
>>> testMyFunction()

# Make changes, reload
>>> .load my-function.nag
>>> testMyFunction()
```

## Next Steps

- **[Language Tutorial](tutorials.md)** - Learn Nagari syntax
- **[API Reference](api-reference.md)** - Explore built-in functions
- **[CLI Reference](cli-reference.md)** - Command-line options
- **[Examples](../examples/)** - Real-world code samples

---

*Master the REPL to accelerate your Nagari development workflow!*

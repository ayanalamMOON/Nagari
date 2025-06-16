# Nagari Programming Language

A modern, Pythonic scripting language that transpiles to JavaScript, designed for full-stack developers who want Python-like syntax with JavaScript ecosystem compatibility.

## Features

- **Pythonic Syntax**: Clean, indentation-based syntax inspired by Python
- **JavaScript Transpilation**: Compiles to clean, readable JavaScript (ES6+)
- **Full JS Ecosystem**: Compatible with React, Vue, Express, and all npm packages
- **Static + Dynamic Typing**: Optional type annotations with runtime flexibility
- **Async-First**: Native support for async/await patterns
- **JSX Support**: First-class support for React JSX syntax
- **No Semicolons**: Clean syntax without unnecessary punctuation
- **Universal**: Works in browsers, Node.js, and web frameworks

## Quick Start

### Hello World

```nag
def greet(name: str = "world") -> str:
    return "Hello, " + name + "!"

print(greet())
```

### React Component

```nag
import React from "react"

def HelloComponent(props):
    return <div>Hello, {props.name}!</div>

export default HelloComponent
```

### Async Example

```nag
async def fetch_data():
    data = await fetch("https://api.example.com/data")
    return data.json()

async def main():
    result = await fetch_data()
    console.log(result)
```

## Project Structure

```text
nagari/
├── nagari-compiler/    # Rust-based lexer, parser, and JS transpiler
├── nagari-runtime/     # Runtime utilities and polyfills
├── stdlib/             # Standard library (transpiles to JS)
├── examples/           # Sample Nagari programs
├── specs/              # Language specification
├── tools/              # Development tools (bundler, dev server)
└── docs/               # Documentation
```

## Installation

```bash
# Install via npm (coming soon)
npm install -g nagari

# Or build from source
git clone https://github.com/nagari-lang/nagari.git
cd nagari
cargo build --release
```

## Usage

```bash
# Transpile to JavaScript
nagc src/app.nag --output dist/app.js

# Development with watch mode
nagc src/ --output dist/ --watch

# Build for production with minification
nagc src/ --output dist/ --minify --bundle

# Generate TypeScript declarations
nagc src/ --output dist/ --declarations

# Build for specific targets
nagc src/ --output dist/ --target browser  # For web
nagc src/ --output dist/ --target node     # For Node.js
```

## Language Specification

- **File Extension**: `.nag`
- **Indentation**: 4 spaces (strict)
- **Type System**: Optional static typing with runtime flexibility
- **Async Support**: Native async/await with Promise integration
- **Interop**: Seamless JavaScript library integration
- **JSX**: First-class React component support

## Documentation

Comprehensive documentation is available:

- **[Getting Started](docs/getting-started.md)**: Installation and first steps
- **[Language Specification](specs/language-spec.md)**: Complete language reference
- **[API Reference](docs/api-reference.md)**: Built-in functions and standard library
- **[Tutorials](docs/tutorials.md)**: Step-by-step learning guides
- **[JavaScript Interop Guide](docs/interop-guide.md)**: Working with JS libraries
- **[Development Guide](docs/development-guide.md)**: Contributing to Nagari
- **[Troubleshooting](docs/troubleshooting.md)**: Common issues and solutions
- **[FAQ](docs/faq.md)**: Frequently asked questions

## Example Projects

The `examples/` directory contains comprehensive demonstrations:

- **[CLI Application](examples/cli_demo.nag)**: Command-line tool with argument parsing
- **[Express Web Server](examples/web_server.nag)**: RESTful API with middleware
- **[React Todo App](examples/react_todo_app.nag)**: Interactive web application
- **[Vue Task Manager](examples/vue_task_app.nag)**: Vue.js integration example
- **[JavaScript Interop](examples/js_interop_demo.nag)**: Library integration patterns

## License

MIT License

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests for any improvements.

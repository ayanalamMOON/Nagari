# Nagari Documentation Index

Welcome to the comprehensive documentation for the Nagari programming language. This index provides access to all available documentation organized by category.

## Quick Start

- **[README](../README.md)** - Project overview and quick introduction
- **[Getting Started](getting-started.md)** - Installation and first steps
- **[Tutorial 1: Hello World](tutorials.md#tutorial-1-hello-world-and-basic-syntax)** - Your first Nagari program

## Language Reference

### Core Documentation

- **[Language Specification](../specs/language-spec.md)** - Complete language reference
- **[Grammar Definition](../specs/grammar.bnf)** - Formal grammar in BNF notation
- **[API Reference](api-reference.md)** - Built-in functions and standard library

### Learning Resources

- **[Tutorials](tutorials.md)** - Comprehensive step-by-step tutorials
  - Basic syntax and control flow
  - Functions and object-oriented programming
  - Async programming and JavaScript interop
  - React components and web development
  - CLI applications and server-side development
- **[Examples](../examples/)** - Working code examples
  - [Hello World](../examples/hello.nag)
  - [Math Operations](../examples/math_demo.nag)
  - [Async/Await](../examples/async_demo.nag)
  - [React Components](../examples/react_component.nag)
  - [Express Server](../examples/express_server.nag)

## Integration Guides

### JavaScript Ecosystem

- **[JavaScript Interop Guide](interop-guide.md)** - Working with JS libraries
- **[React Integration](interop-guide.md#react-jsx-support)** - Building React apps with Nagari
- **[Node.js Development](tutorials.md#tutorial-8-server-side-development)** - Server-side Nagari

### Runtime Support

- **[Bun Integration Guide](bun-guide.md)** - 🚀 Blazing-fast Bun runtime support _(New!)_
  - 4x faster performance
  - Automatic runtime detection
  - Installation and setup
  - Performance benchmarks
  - Best practices

### Development Tools

- **[CLI Reference](cli-reference.md)** - Command-line interface documentation
- **[VS Code Extension](../tools/vscode-extension/)** - IDE support and features
- **[Build Tools](../tools/)** - Build scripts and automation

## Development

### Contributing

- **[Contributing Guide](../CONTRIBUTING.md)** - How to contribute to Nagari
- **[Development Guide](development-guide.md)** - Developer documentation
- **[Code of Conduct](../CODE_OF_CONDUCT.md)** - Community guidelines

### Project Information

- **[Changelog](../CHANGELOG.md)** - Version history and changes
- **[License](../LICENSE)** - MIT License terms
- **[Roadmap](development-guide.md#future-development)** - Future development plans

## Support

### Help and Troubleshooting

- **[FAQ](faq.md)** - Frequently asked questions
- **[Troubleshooting Guide](troubleshooting.md)** - Common issues and solutions
- **[Performance Guide](development-guide.md#performance-considerations)** - Optimization tips

### Community

- **[GitHub Issues](https://github.com/nagari-lang/nagari/issues)** - Bug reports and feature requests
- **[GitHub Discussions](https://github.com/nagari-lang/nagari/discussions)** - Community discussions
- **[Discord Server](https://discord.gg/nagari)** - Real-time chat and support

## Architecture Documentation

### Compiler Architecture

- **[Lexer](../nagari-compiler/src/lexer.rs)** - Tokenization and lexical analysis
- **[Parser](../nagari-compiler/src/parser.rs)** - AST generation and syntax parsing
- **[Type System](../nagari-compiler/src/types.rs)** - Type checking and inference
- **[Transpiler](../nagari-compiler/src/transpiler/)** - JavaScript code generation

### Runtime Architecture

- **[Runtime Library](../nagari-runtime/)** - TypeScript runtime and polyfills
- **[Interop System](../nagari-runtime/src/interop.ts)** - JavaScript integration
- **[Built-ins](../nagari-runtime/src/builtins.ts)** - Core function implementations

### Standard Library

- **[Core](../stdlib/core.nag)** - Essential utilities and data structures
- **[HTTP](../stdlib/http.nag)** - HTTP client and server utilities
- **[File System](../stdlib/fs.nag)** - File and directory operations
- **[Math](../stdlib/math.nag)** - Mathematical functions and constants
- **[JSON](../stdlib/json.nag)** - JSON parsing and serialization
- **[Time](../stdlib/time.nag)** - Date and time utilities
- **[OS](../stdlib/os.nag)** - Operating system interface
- **[Database](../stdlib/db.nag)** - Database connectivity
- **[Crypto](../stdlib/crypto.nag)** - Cryptographic functions

## Examples by Category

### Basic Examples

- **[Hello World](../examples/hello.nag)** - Simple greeting program
- **[Math Demo](../examples/math_demo.nag)** - Mathematical operations
- **[Algorithms](../examples/algorithms.nag)** - Common algorithms implementation

### Async Programming

- **[Async Demo](../examples/async_demo.nag)** - Basic async/await patterns
- **[Fetch Demo](../examples/fetch_demo.nag)** - HTTP requests and API calls
- **[File Operations](../examples/file_operations.nag)** - Async file handling

### Web Development

- **[React Component](../examples/react_component.nag)** - Simple React component
- **[React Todo App](../examples/react_todo_app.nag)** - Complete React application
- **[Vue Task App](../examples/vue_task_app.nag)** - Vue.js integration
- **[Express Server](../examples/express_server.nag)** - REST API server
- **[Web Server](../examples/web_server.nag)** - HTTP server with routing

### JavaScript Integration

- **[Interop Demo](../examples/interop_demo.nag)** - Basic JS interop patterns
- **[JS Interop Demo](../examples/js_interop_demo.nag)** - Advanced interop techniques

### Command Line Applications

- **[CLI Demo](../examples/cli_demo.nag)** - Command-line interface example

## Testing Documentation

### Test Structure

- **[Test Overview](../tools/test-toolchain.sh)** - Comprehensive test script
- **[Example Tests](../tools/test-examples.sh)** - Example validation script
- **[Unit Tests](../nagari-compiler/tests/)** - Compiler unit tests
- **[Runtime Tests](../nagari-runtime/__tests__/)** - Runtime library tests

### Testing Guides

- **[Writing Tests](development-guide.md#testing)** - How to write good tests
- **[Testing Strategy](development-guide.md#test-coverage)** - Testing approach and coverage
- **[Debugging Tests](troubleshooting.md#debugging-techniques)** - Debugging test failures

## Specifications

### Language Specifications

- **[Bytecode Format](../specs/bytecode-format.md)** - VM bytecode specification (legacy)
- **[Grammar](../specs/grammar.bnf)** - Complete language grammar
- **[Type System](api-reference.md#data-types)** - Type system documentation

## Build and Deployment

### Build System

- **[Build Script](../tools/build.sh)** - Unix/Linux build automation
- **[Build Script (Windows)](../tools/build.bat)** - Windows build automation
- **[Cargo Configuration](../nagari-compiler/Cargo.toml)** - Rust build configuration
- **[NPM Configuration](../nagari-runtime/package.json)** - Node.js build configuration

### Deployment Guides

- **[Web Deployment](faq.md#how-do-i-deploy-nagari-applications)** - Deploying web applications
- **[Server Deployment](tutorials.md#tutorial-8-server-side-development)** - Server deployment strategies
- **[CI/CD](development-guide.md#release-process)** - Continuous integration and deployment

## Version Information

- **Current Version**: 0.2.0-dev
- **Compatibility**: JavaScript ES6+, Node.js 16+
- **License**: MIT
- **Repository**: [GitHub](https://github.com/nagari-lang/nagari)

## Documentation Conventions

### Formatting

- **Code blocks**: Syntax-highlighted Nagari, JavaScript, and shell code
- **File paths**: Relative to repository root
- **Commands**: Executable shell commands with expected output
- **Links**: Internal and external references

### Symbols

- 🚀 **New Feature**: Recently added functionality
- ⚠️ **Warning**: Important caveats or breaking changes
- 💡 **Tip**: Helpful suggestions and best practices
- 🐛 **Bug**: Known issues and workarounds

---

**Last Updated**: January 2024
**Contributors**: See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines

For questions or suggestions about the documentation, please open an issue on [GitHub](https://github.com/nagari-lang/nagari/issues) or join our [Discord community](https://discord.gg/nagari).

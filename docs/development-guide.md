# Nagari Development Guide

This guide covers how to contribute to the Nagari programming language project, including setting up the development environment, understanding the architecture, and implementing new features.

## Table of Contents

1. [Development Setup](#development-setup)
2. [Project Architecture](#project-architecture)
3. [Compiler Architecture](#compiler-architecture)
4. [Runtime Architecture](#runtime-architecture)
5. [Testing](#testing)
6. [Contributing Guidelines](#contributing-guidelines)
7. [Release Process](#release-process)

## Development Setup

### Prerequisites

- **Rust**: 1.70 or later (with Cargo)
- **Node.js**: 16 or later (with npm)
- **Git**: For version control
- **TypeScript**: For runtime development

### Environment Setup

1. **Clone the repository:**

```bash
git clone https://github.com/nagari-lang/nagari.git
cd nagari
```

2. **Build the compiler:**

```bash
cd nagari-compiler
cargo build --debug  # For development
cargo build --release  # For production
```

3. **Install runtime dependencies:**

```bash
cd ../nagari-runtime
npm install
npm run build
```

4. **Run tests:**

```bash
# Test the entire toolchain
./tools/test-toolchain.sh

# Test specific examples
./tools/test-examples.sh
```

### Development Tools

- **VS Code Extensions**: Rust-analyzer, TypeScript Hero
- **Debugging**: Use `cargo test` and `npm test` for unit tests
- **Linting**: `cargo clippy` for Rust, `npm run lint` for TypeScript

## Project Architecture

The Nagari project consists of several key components:

```
nagari/
├── nagari-compiler/    # Rust-based compiler (lexer → parser → AST → transpiler)
├── nagari-runtime/     # TypeScript runtime library for JS interop
├── stdlib/             # Standard library written in Nagari
├── examples/           # Example programs and use cases
├── specs/              # Language specification and grammar
├── tools/              # Build scripts and development utilities
└── docs/               # Documentation
```

### Core Components

1. **Compiler (`nagari-compiler/`)**:
   - Lexical analysis (tokenization)
   - Syntax parsing (AST generation)
   - Type checking and analysis
   - JavaScript transpilation

2. **Runtime (`nagari-runtime/`)**:
   - JavaScript interop utilities
   - Built-in function polyfills
   - Type conversion helpers
   - React/JSX support

3. **Standard Library (`stdlib/`)**:
   - Core utilities and data structures
   - HTTP, filesystem, and OS modules
   - Math, crypto, and time utilities

## Compiler Architecture

The Nagari compiler is built in Rust and follows a traditional multi-stage pipeline:

### Compilation Pipeline

```
Source Code (.nag)
    ↓
Lexer (src/lexer.rs)
    ↓
Tokens
    ↓
Parser (src/parser.rs)
    ↓
AST (src/ast.rs)
    ↓
Type Checker (src/types.rs)
    ↓
Transpiler (src/transpiler/)
    ↓
JavaScript Code (.js)
```

### Key Files

- **`src/main.rs`**: CLI entry point and argument parsing
- **`src/lexer.rs`**: Tokenization and lexical analysis
- **`src/parser.rs`**: Recursive descent parser for AST generation
- **`src/ast.rs`**: Abstract syntax tree definitions
- **`src/types.rs`**: Type system and inference
- **`src/transpiler/`**: JavaScript code generation

### Transpiler Architecture

The transpiler is modular and extensible:

- **`mod.rs`**: Main transpiler coordination
- **`modules.rs`**: Module resolution and import handling
- **`js_runtime.rs`**: Runtime helper injection
- **`builtin_map.rs`**: Built-in function mapping

### Adding New Language Features

1. **Update the lexer** (`src/lexer.rs`):
   - Add new token types if needed
   - Update tokenization logic

2. **Update the parser** (`src/parser.rs`):
   - Add new AST node parsing
   - Update grammar rules

3. **Update the AST** (`src/ast.rs`):
   - Define new node types
   - Add visitor patterns if needed

4. **Update the transpiler** (`src/transpiler/mod.rs`):
   - Add JavaScript generation for new constructs
   - Update runtime helpers if needed

5. **Add tests**:
   - Unit tests in the respective modules
   - Integration tests in `examples/`

## Runtime Architecture

The runtime is built in TypeScript and provides:

### Core Modules

- **`src/index.ts`**: Main runtime exports
- **`src/builtins.ts`**: Built-in function implementations
- **`src/types.ts`**: Type definitions and utilities
- **`src/async.ts`**: Async/await support
- **`src/interop.ts`**: JavaScript interop utilities
- **`src/jsx.ts`**: React/JSX support

### Interop System

The interop system allows seamless communication between Nagari and JavaScript:

```typescript
// Value conversion
const nagarifiedValue = nagari.from_js(jsValue);
const jsValue = nagari.to_js(nagarifiedValue);

// Function wrapping
const nagariFn = nagari.wrap_js_function(jsFn);
const jsFn = nagari.unwrap_function(nagariFn);

// Module wrapping
const nagariModule = nagari.wrap_js_module(jsModule);
```

### Adding Runtime Features

1. **Update type definitions** (`src/types.ts`)
2. **Implement functionality** (appropriate module)
3. **Add to exports** (`src/index.ts`)
4. **Update interop** if needed (`src/interop.ts`)
5. **Add tests** (`__tests__/`)

## Testing

### Test Structure

```
tests/
├── unit/           # Unit tests for individual components
├── integration/    # Integration tests for full pipeline
├── examples/       # Example-based tests
└── benchmarks/     # Performance benchmarks
```

### Running Tests

```bash
# Run all tests
./tools/test-toolchain.sh

# Run compiler tests only
cd nagari-compiler && cargo test

# Run runtime tests only
cd nagari-runtime && npm test

# Run specific test
cargo test lexer_tests
npm test -- --testNamePattern="interop"
```

### Test Coverage

- **Unit Tests**: Test individual functions and modules
- **Integration Tests**: Test complete compilation pipeline
- **Example Tests**: Validate example programs compile and run
- **Interop Tests**: Test JavaScript integration

### Writing Tests

#### Compiler Tests (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_keywords() {
        let mut lexer = Lexer::new("def function():");
        assert_eq!(lexer.next_token().token_type, TokenType::Def);
        assert_eq!(lexer.next_token().token_type, TokenType::Identifier);
    }
}
```

#### Runtime Tests (TypeScript/Jest)

```typescript
describe('Interop', () => {
    test('should convert JS values to Nagari', () => {
        const result = nagari.from_js([1, 2, 3]);
        expect(result.type).toBe('list');
        expect(result.value).toEqual([1, 2, 3]);
    });
});
```

## Contributing Guidelines

### Code Style

#### Rust Code

- Follow `rustfmt` formatting
- Use `cargo clippy` for linting
- Write comprehensive documentation comments
- Prefer explicit error handling over panics

#### TypeScript Code

- Follow Prettier formatting
- Use ESLint for linting
- Write JSDoc comments for public APIs
- Prefer explicit types over `any`

### Commit Messages

Follow conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:

- `feat(parser): add pattern matching support`
- `fix(transpiler): handle async function edge cases`
- `docs(api): update function reference`

### Pull Request Process

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/new-feature`
3. **Make changes and add tests**
4. **Run the test suite**: `./tools/test-toolchain.sh`
5. **Update documentation** if needed
6. **Submit pull request** with clear description

### Review Criteria

- **Functionality**: Does it work as intended?
- **Tests**: Are there adequate tests?
- **Documentation**: Is it properly documented?
- **Performance**: Does it introduce performance regressions?
- **Compatibility**: Does it break existing functionality?

## Release Process

### Version Numbering

Nagari follows semantic versioning (semver):

- **Major**: Breaking changes to language or API
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes, backward compatible

### Release Checklist

1. **Update version numbers**:
   - `nagari-compiler/Cargo.toml`
   - `nagari-runtime/package.json`
   - Documentation versions

2. **Run full test suite**:

   ```bash
   ./tools/test-toolchain.sh
   ./tools/test-examples.sh
   ```

3. **Update changelog**: Document all changes since last release

4. **Build release artifacts**:

   ```bash
   cargo build --release
   cd nagari-runtime && npm run build
   ```

5. **Tag the release**:

   ```bash
   git tag -a v0.1.0 -m "Release version 0.1.0"
   git push origin v0.1.0
   ```

6. **Publish packages**:

   ```bash
   # Publish to crates.io (if applicable)
   cargo publish

   # Publish to npm
   cd nagari-runtime && npm publish
   ```

### Release Types

- **Alpha**: Early development, breaking changes expected
- **Beta**: Feature complete, bug fixes only
- **RC (Release Candidate)**: Production ready, final testing
- **Stable**: Production release

## Performance Considerations

### Compiler Performance

- **Incremental Compilation**: Cache AST and type information
- **Parallel Processing**: Parallelize independent modules
- **Memory Management**: Minimize allocations in hot paths

### Runtime Performance

- **Minimal Overhead**: Keep interop costs low
- **Lazy Loading**: Load modules and features on demand
- **Optimized Transpilation**: Generate efficient JavaScript

### Benchmarking

```bash
# Run performance benchmarks
cd benchmarks
./run-benchmarks.sh

# Profile compilation
cargo flamegraph --bin nagc -- examples/large_file.nag

# Profile runtime
cd nagari-runtime
npm run benchmark
```

## Debugging

### Compiler Debugging

```bash
# Enable debug output
RUST_LOG=debug cargo run -- examples/debug.nag

# Use rust-gdb for debugging
rust-gdb target/debug/nagc

# Print AST for debugging
cargo run -- examples/test.nag --print-ast
```

### Runtime Debugging

```javascript
// Enable debug mode
process.env.NAGARI_DEBUG = 'true';

// Use Node.js inspector
node --inspect-brk compiled_output.js

// Debug transpiled output
nagc examples/test.nag --output debug.js --sourcemap
```

## Future Development

### Planned Features

- **Pattern Matching**: Advanced pattern matching syntax
- **Generics**: Generic types and functions
- **Macros**: Compile-time code generation
- **Native Compilation**: Optional native backend
- **IDE Support**: Language server protocol implementation

### Architecture Improvements

- **Incremental Compilation**: Faster rebuild times
- **Better Error Messages**: More helpful diagnostics
- **Plugin System**: Extensible compiler architecture
- **WASM Backend**: WebAssembly compilation target

This development guide should help you get started contributing to Nagari. For questions or clarifications, please open an issue or join our community discussions.

# Tests Directory

This directory contains test files and debugging utilities for the Nagari language toolchain.

## Structure

- `fixtures/` - Test Nagari source files (.nag)
- `outputs/` - Generated JavaScript files and other compilation outputs
- `debug/` - Debug utilities and temporary files for development
- `integration/` - Integration test suites (future)
- `unit/` - Unit test files (future)

## Usage

- **fixtures/**: Contains sample Nagari programs used for testing various language features
- **outputs/**: Generated files from compilation, useful for verifying transpiler output
- **debug/**: Tools for debugging lexer, parser, and compiler issues

## Running Tests

```bash
# Run all tests
cargo test

# Test a specific fixture
cargo run --bin nag -- run tests/fixtures/test_simple_function.nag

# Compile a fixture to JavaScript
cargo run --bin nagc -- tests/fixtures/test_simple_function.nag --output tests/outputs/test_simple_function.js
```

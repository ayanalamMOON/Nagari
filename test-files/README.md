# Test Files Organization

This directory contains various test files and temporary artifacts for the Nagari language project.

## Directory Structure

### `samples/`
Contains `.nag` test files that demonstrate various language features and syntax:
- Basic syntax tests
- Function tests
- Async arrow function tests
- Math operation tests
- String handling tests
- Type system tests

### `integration/`
Contains integration test files in various languages:
- Rust integration tests (`.rs` files)
- JavaScript test files (`.js` files)
- Cross-language interop tests

### `temp/`
Contains temporary test files and configurations:
- Temporary Cargo.toml configurations
- Build artifacts from testing
- Other temporary testing resources

### `test_parser_project/`
Contains a separate test project for parser development and testing.

## Usage

These test files are used during development to validate language features and ensure proper compilation. Most files in `samples/` can be compiled using the Nagari compiler to verify syntax and semantics.

## Note

Files in the `temp/` directory are typically build artifacts or temporary configurations and may be cleaned up periodically.

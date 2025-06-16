# Nagari CLI Integration Guide

This guide covers the complete Nagari CLI toolchain and how to integrate it into your development workflow.

## Table of Contents

1. [Installation & Setup](#installation--setup)
2. [Project Management](#project-management)
3. [Development Workflow](#development-workflow)
4. [Build & Deployment](#build--deployment)
5. [Code Quality Tools](#code-quality-tools)
6. [Package Management](#package-management)
7. [Developer Tools](#developer-tools)
8. [Editor Integration](#editor-integration)
9. [CI/CD Integration](#cicd-integration)
10. [Troubleshooting](#troubleshooting)

## Installation & Setup

### Building the CLI

```bash
# Clone the repository
git clone https://github.com/nagari-lang/nagari.git
cd nagari

# Build the CLI tool
cargo build --release --manifest-path cli/Cargo.toml

# Install globally (optional)
cargo install --path cli/
```

### Verify Installation

```bash
nag --version
nag --help
```

## Project Management

### Creating New Projects

```bash
# Basic project
nag init my-project --template basic

# Web application
nag init my-web-app --template web

# CLI application
nag init my-cli-tool --template cli

# Library/package
nag init my-library --template library

# Interactive mode
nag init my-project
```

### Project Structure

my-project/
├── nagari.toml          # Project configuration
├── main.nag            # Entry point
├── src/                # Source files
├── tests/              # Test files
├── docs/               # Documentation
├── .gitignore          # Git ignore rules
└── nagari.json         # Package metadata (if using package manager)

### Configuration (nagari.toml)

```toml
[project]
name = "my-project"
version = "0.1.0"
description = "My Nagari project"
main = "main.nag"
authors = ["Your Name <your.email@example.com>"]

[build]
target = "js"           # js, bytecode, wasm
output_dir = "dist"
optimization = true
sourcemap = true

[format]
indent_size = 4
max_line_length = 100
trailing_commas = true

[lint]
max_complexity = 10
unused_variables = "warn"
style_issues = "error"

[package]
registry = "https://packages.nagari-lang.org"
```

## Development Workflow

### Running Code

```bash
# Run a single file
nag run main.nag

# Run with arguments
nag run main.nag -- --verbose --input file.txt

# Watch mode (auto-restart on changes)
nag run main.nag --watch

# Run from any directory
nag run src/cli.nag
```

### Interactive Development

```bash
# Start REPL
nag repl

# Load a script in REPL
nag repl --script utils.nag

# Enable experimental features
nag repl --experimental
```

### Development Server

```bash
# Start dev server (for web projects)
nag serve

# Custom port and entry point
nag serve --port 8080 --entry src/app.nag

# Enable HTTPS
nag serve --https

# Serve static files
nag serve --public ./public
```

## Build & Deployment

### Basic Building

```bash
# Build current project
nag build main.nag

# Build to specific directory
nag build src/ --output dist/

# Build for production
nag build main.nag --release
```

### Transpilation

```bash
# Transpile to JavaScript
nag transpile main.nag --format esm

# Transpile with type definitions
nag transpile src/ --declarations

# Minify output
nag transpile main.nag --minify
```

### Bundling

```bash
# Create bundle
nag bundle main.nag --output app.js

# Tree-shaking
nag bundle main.nag --treeshake

# External dependencies
nag bundle main.nag --external lodash,react

# Different formats
nag bundle main.nag --format cjs    # CommonJS
nag bundle main.nag --format esm    # ES Modules
nag bundle main.nag --format iife   # Browser global
```

### Multiple Targets

```bash
# Build for different targets
nag build main.nag --target js
nag build main.nag --target bytecode
nag build main.nag --target wasm
```

## Code Quality Tools

### Formatting

```bash
# Format all files
nag format src/

# Check formatting without modifying
nag format --check src/

# Show diff
nag format --diff src/

# Format specific files
nag format main.nag utils.nag
```

### Linting

```bash
# Lint project
nag lint src/

# Auto-fix issues
nag lint src/ --fix

# Different output formats
nag lint src/ --format json
nag lint src/ --format text

# Lint specific files
nag lint main.nag --fix
```

### Testing

```bash
# Run all tests
nag test

# Run specific test files
nag test tests/math.nag

# Pattern matching
nag test --pattern "*unit*"

# Coverage reporting
nag test --coverage

# Watch mode
nag test --watch
```

## Package Management

### Initialization

```bash
# Initialize package configuration
nag package init

# Non-interactive mode
nag package init --yes
```

### Managing Dependencies

```bash
# Install packages
nag package install requests numpy

# Install development dependencies
nag package install --dev pytest mypy

# Install global packages
nag package install --global nagari-tools

# Install exact versions
nag package install requests@2.28.0 --exact

# Add single package
nag package add express@4.18.0

# Remove packages
nag package remove lodash express

# Update packages
nag package update
nag package update requests numpy
```

### Package Information

```bash
# List installed packages
nag package list

# Tree view
nag package list --tree

# Check for outdated packages
nag package list --outdated
```

### Publishing

```bash
# Pack package for distribution
nag package pack

# Publish to registry
nag package publish

# Dry run (test publishing)
nag package publish --dry-run

# Publish to specific registry
nag package publish --registry https://my-registry.com
```

## Developer Tools

### Documentation

```bash
# Generate documentation
nag doc generate --source src/ --output docs/

# Include private members
nag doc generate --source src/ --private

# Different formats
nag doc generate --source src/ --format html
nag doc generate --source src/ --format markdown

# Serve documentation locally
nag doc serve --port 8080

# Check documentation coverage
nag doc check docs/
```

### Language Server

```bash
# Start LSP server (for editor integration)
nag lsp stdio

# TCP mode
nag lsp tcp --port 9257

# WebSocket mode
nag lsp websocket --port 9258
```

## Editor Integration

### VS Code

1. Install the Nagari extension from the marketplace
2. The LSP server will start automatically
3. Configure in settings:

```json
{
  "nagari.lsp.enabled": true,
  "nagari.lsp.path": "/path/to/nag",
  "nagari.format.onSave": true,
  "nagari.lint.onSave": true
}
```

### Vim/Neovim

```lua
-- LSP configuration for Neovim
require'lspconfig'.nagari.setup{
  cmd = {"nag", "lsp", "stdio"},
  filetypes = {"nagari"},
  root_dir = require'lspconfig'.util.root_pattern("nagari.toml", "nagari.json"),
}
```

### Other Editors

Configure your editor to:

1. Run `nag lsp stdio` for LSP features
2. Use `nag format` for formatting
3. Use `nag lint` for linting
4. Set file association for `.nag` files

## CI/CD Integration

### GitHub Actions

```yaml
name: Nagari CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build Nagari CLI
        run: cargo build --release --manifest-path cli/Cargo.toml

      - name: Install Nagari CLI
        run: cargo install --path cli/

      - name: Check formatting
        run: nag format --check src/

      - name: Run linter
        run: nag lint src/

      - name: Run tests
        run: nag test --coverage

      - name: Build project
        run: nag build src/ --release
```

### Jenkins

```groovy
pipeline {
    agent any

    stages {
        stage('Setup') {
            steps {
                sh 'cargo build --release --manifest-path cli/Cargo.toml'
                sh 'cargo install --path cli/'
            }
        }

        stage('Quality Checks') {
            parallel {
                stage('Format') {
                    steps {
                        sh 'nag format --check src/'
                    }
                }
                stage('Lint') {
                    steps {
                        sh 'nag lint src/'
                    }
                }
            }
        }

        stage('Test') {
            steps {
                sh 'nag test --coverage'
            }
        }

        stage('Build') {
            steps {
                sh 'nag build src/ --release'
            }
        }
    }
}
```

### GitLab CI

```yaml
stages:
  - setup
  - quality
  - test
  - build

setup:
  stage: setup
  script:
    - cargo build --release --manifest-path cli/Cargo.toml
    - cargo install --path cli/
  artifacts:
    paths:
      - target/release/nag

format:
  stage: quality
  script:
    - nag format --check src/

lint:
  stage: quality
  script:
    - nag lint src/

test:
  stage: test
  script:
    - nag test --coverage

build:
  stage: build
  script:
    - nag build src/ --release
  artifacts:
    paths:
      - dist/
```

## Troubleshooting

### Common Issues

**CLI not found after installation:**

```bash
# Make sure cargo bin directory is in PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Or use full path
~/.cargo/bin/nag --version
```

**Build failures:**

```bash
# Clean and rebuild
cargo clean --manifest-path cli/Cargo.toml
cargo build --release --manifest-path cli/Cargo.toml

# Check dependencies
cargo check --manifest-path cli/Cargo.toml
```

**LSP server not working:**

```bash
# Test LSP manually
nag lsp stdio

# Check if port is available
nag lsp tcp --port 9258
```

**Permission errors:**

```bash
# Fix executable permissions
chmod +x ~/.cargo/bin/nag

# Or use sudo for global installation
sudo cargo install --path cli/
```

### Debug Mode

```bash
# Enable verbose output
nag --verbose build main.nag

# Use debug configuration
export RUST_LOG=debug
nag build main.nag
```

### Performance Issues

```bash
# Use release mode for better performance
cargo install --path cli/ --release

# Profile build times
nag build main.nag --profile

# Use parallel processing
nag build src/ --parallel
```

### Getting Help

```bash
# General help
nag --help

# Command-specific help
nag build --help
nag package --help

# Check configuration
nag config check

# Diagnostic information
nag doctor
```

## Best Practices

1. **Use configuration files:** Keep project settings in `nagari.toml`
2. **Version control:** Include `nagari.toml` and `nagari.json` in git
3. **Ignore build outputs:** Add `dist/` and `*.js.map` to `.gitignore`
4. **Use watch mode:** Enable `--watch` during development
5. **Automate quality checks:** Set up pre-commit hooks
6. **Keep dependencies updated:** Regularly run `nag package update`
7. **Use consistent formatting:** Run `nag format` before commits
8. **Write tests:** Use `nag test` in your CI pipeline

This integration guide provides a complete overview of using the Nagari CLI toolchain effectively in your development workflow.

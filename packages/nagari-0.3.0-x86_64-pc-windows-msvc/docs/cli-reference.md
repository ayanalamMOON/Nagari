# CLI Reference

Complete command-line interface reference for the Nagari programming language.

## Overview

The Nagari CLI provides a comprehensive set of commands for development, compilation, and package management.

```bash
nagari [COMMAND] [OPTIONS] [ARGS]
```

## Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Show help information |
| `--version` | `-V` | Show version information |
| `--verbose` | `-v` | Enable verbose output |
| `--quiet` | `-q` | Suppress non-error output |
| `--config <FILE>` | `-c` | Use custom config file |

## Commands

### `run` - Execute Nagari Files

Execute a Nagari source file directly.

```bash
nagari run [OPTIONS] <FILE> [ARGS...]
```

**Options:**
- `--runtime <RUNTIME>` - Target runtime (node, browser, deno)
- `--debug` - Enable debug mode
- `--watch` - Watch file for changes and re-run
- `--optimize` - Enable optimizations
- `--output <FILE>` - Specify output file
- `--env <ENV>` - Set environment variables

**Examples:**
```bash
# Run a simple script
nagari run hello.nag

# Run with Node.js runtime
nagari run --runtime node server.nag

# Run with debugging enabled
nagari run --debug app.nag

# Run and watch for changes
nagari run --watch dev.nag

# Pass arguments to the script
nagari run script.nag arg1 arg2
```

### `build` - Compile Nagari Code

Compile Nagari source files to target formats.

```bash
nagari build [OPTIONS] <INPUT>
```

**Options:**
- `--output <DIR>` - Output directory (default: `dist/`)
- `--target <TARGET>` - Target format (js, wasm, native)
- `--optimize` - Enable optimizations
- `--sourcemap` - Generate source maps
- `--minify` - Minify output
- `--watch` - Watch for changes and rebuild

**Examples:**
```bash
# Build to JavaScript
nagari build src/ --target js

# Build with optimizations
nagari build src/ --optimize --minify

# Build and watch for changes
nagari build src/ --watch

# Build to WebAssembly
nagari build src/ --target wasm
```

### `repl` - Interactive Shell

Start an interactive Nagari REPL (Read-Eval-Print Loop).

```bash
nagari repl [OPTIONS]
```

**Options:**
- `--runtime <RUNTIME>` - REPL runtime environment
- `--debug` - Enable debug mode
- `--history <FILE>` - Custom history file location
- `--no-history` - Disable command history
- `--load <FILE>` - Load script on startup

**Examples:**
```bash
# Start basic REPL
nagari repl

# REPL with Node.js runtime
nagari repl --runtime node

# REPL with debug mode
nagari repl --debug

# Load a file on startup
nagari repl --load setup.nag
```

### `init` - Initialize Project

Create a new Nagari project with standard structure.

```bash
nagari init [OPTIONS] [NAME]
```

**Options:**
- `--template <TEMPLATE>` - Project template (basic, web, cli, library)
- `--runtime <RUNTIME>` - Default runtime target
- `--git` - Initialize git repository
- `--examples` - Include example files

**Examples:**
```bash
# Create basic project
nagari init my-project

# Create web application
nagari init --template web my-web-app

# Create with git and examples
nagari init --git --examples my-project
```

### `test` - Run Tests

Execute test suites and individual test files.

```bash
nagari test [OPTIONS] [PATTERN]
```

**Options:**
- `--pattern <PATTERN>` - Test file pattern (default: `**/*.test.nag`)
- `--watch` - Watch for changes and re-run tests
- `--coverage` - Generate coverage report
- `--reporter <REPORTER>` - Test reporter (spec, json, junit)
- `--timeout <MS>` - Test timeout in milliseconds

**Examples:**
```bash
# Run all tests
nagari test

# Run specific test pattern
nagari test "**/*.spec.nag"

# Run with coverage
nagari test --coverage

# Watch mode for TDD
nagari test --watch
```

### `install` - Package Management

Install and manage dependencies.

```bash
nagari install [OPTIONS] [PACKAGE]
```

**Options:**
- `--dev` - Install as development dependency
- `--global` - Install globally
- `--save` - Save to package.nag (default)
- `--version <VERSION>` - Specific version to install

**Examples:**
```bash
# Install all dependencies
nagari install

# Install specific package
nagari install math@1.0.0

# Install as dev dependency
nagari install --dev testing-framework
```

### `publish` - Package Publishing

Publish packages to the Nagari registry.

```bash
nagari publish [OPTIONS]
```

**Options:**
- `--registry <URL>` - Custom registry URL
- `--tag <TAG>` - Publish with specific tag
- `--dry-run` - Simulate publishing without uploading
- `--access <LEVEL>` - Package access level (public, private)

**Examples:**
```bash
# Publish to default registry
nagari publish

# Dry run to check package
nagari publish --dry-run

# Publish with specific tag
nagari publish --tag beta
```

### `format` - Code Formatting

Format Nagari source code according to style guidelines.

```bash
nagari format [OPTIONS] [FILES...]
```

**Options:**
- `--check` - Check if files are formatted without modifying
- `--write` - Write formatted output to files (default)
- `--diff` - Show differences without applying changes
- `--config <FILE>` - Custom formatting configuration

**Examples:**
```bash
# Format all .nag files
nagari format src/**/*.nag

# Check formatting without changes
nagari format --check src/

# Show diff of changes
nagari format --diff main.nag
```

### `lint` - Code Analysis

Analyze code for style and potential issues.

```bash
nagari lint [OPTIONS] [FILES...]
```

**Options:**
- `--fix` - Automatically fix issues where possible
- `--rules <RULES>` - Specify linting rules
- `--config <FILE>` - Custom lint configuration
- `--format <FORMAT>` - Output format (text, json, junit)

**Examples:**
```bash
# Lint all source files
nagari lint src/

# Lint with auto-fix
nagari lint --fix src/

# Custom output format
nagari lint --format json src/
```

### `lsp` - Language Server

Start the Nagari Language Server Protocol implementation.

```bash
nagari lsp [OPTIONS]
```

**Options:**
- `--stdio` - Use stdio for communication (default)
- `--tcp <PORT>` - Use TCP on specified port
- `--debug` - Enable debug logging
- `--log-file <FILE>` - Write logs to file

**Examples:**
```bash
# Start LSP server
nagari lsp

# LSP with TCP communication
nagari lsp --tcp 9257

# LSP with debug logging
nagari lsp --debug --log-file lsp.log
```

### `doc` - Documentation

Generate and serve documentation.

```bash
nagari doc [OPTIONS] [COMMAND]
```

**Subcommands:**
- `generate` - Generate documentation from source
- `serve` - Serve documentation locally
- `build` - Build static documentation site

**Examples:**
```bash
# Generate documentation
nagari doc generate src/

# Serve documentation locally
nagari doc serve --port 8080

# Build static site
nagari doc build --output docs-site/
```

## Configuration

### Config File

Nagari uses `nagari.toml` for configuration:

```toml
[build]
target = "js"
optimize = true
sourcemap = true

[runtime]
default = "node"

[lint]
rules = ["standard"]
auto-fix = true

[test]
timeout = 5000
coverage = true
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `NAGARI_HOME` | Nagari installation directory |
| `NAGARI_REGISTRY` | Default package registry URL |
| `NAGARI_CACHE` | Cache directory location |
| `NAGARI_DEBUG` | Enable debug output |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | File not found |
| 4 | Compilation error |
| 5 | Runtime error |
| 6 | Test failure |

## Examples

### Development Workflow

```bash
# Create new project
nagari init my-app --template web --git

# Install dependencies
cd my-app
nagari install

# Start development
nagari run --watch src/main.nag

# Run tests
nagari test --watch

# Build for production
nagari build --optimize --minify
```

### Package Development

```bash
# Create library project
nagari init my-lib --template library

# Add tests
nagari test --coverage

# Format and lint
nagari format src/
nagari lint --fix src/

# Publish package
nagari publish --dry-run
nagari publish
```

## Getting Help

- Use `nagari --help` for general help
- Use `nagari <command> --help` for command-specific help
- Visit [documentation](index.md) for detailed guides
- Check [troubleshooting](troubleshooting.md) for common issues

---

*For more examples and advanced usage, see the [tutorials](tutorials.md).*

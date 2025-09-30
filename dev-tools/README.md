# Nagari Development Tools

This directory contains comprehensive development tools for the Nagari programming language project.

## 🛠️ Available Tools

### Core Development
- `setup-dev-env.sh/bat` - Complete development environment setup
- `dev-server.sh/bat` - Development server with hot reload
- `lint-check.sh/bat` - Code quality and style checking
- `format-code.sh/bat` - Automatic code formatting
- `pre-commit.sh/bat` - Pre-commit hooks and validation

### Testing & Quality
- `test-runner.sh/bat` - Comprehensive test suite runner
- `coverage-report.sh/bat` - Generate test coverage reports
- `benchmark.sh/bat` - Performance benchmarking tools
- `integration-test.sh/bat` - Integration testing suite
- `stress-test.sh/bat` - Stress testing tools

### Build & Release
- `clean-build.sh/bat` - Clean build from scratch
- `release-prep.sh/bat` - Prepare release packages
- `version-bump.sh/bat` - Automated version management
- `package-dist.sh/bat` - Package distribution files
- `deploy-docs.sh/bat` - Documentation deployment

### Development Utilities
- `debug-transpiler.sh/bat` - Transpiler debugging tools
- `profile-compiler.sh/bat` - Compiler performance profiling
- `validate-examples.sh/bat` - Validate all example files
- `check-dependencies.sh/bat` - Dependency health check
- `generate-api-docs.sh/bat` - Generate API documentation

### IDE & Editor Tools
- `setup-vscode.sh/bat` - VS Code development setup
- `update-language-server.sh/bat` - LSP development tools
- `generate-snippets.sh/bat` - Code snippet generation
- `syntax-highlighting.sh/bat` - Syntax highlighting tools

## 🚀 Quick Start

1. **Setup Development Environment**:
   ```bash
   ./dev-tools/setup-dev-env.sh
   ```

2. **Start Development Server**:
   ```bash
   ./dev-tools/dev-server.sh
   ```

3. **Run Tests**:
   ```bash
   ./dev-tools/test-runner.sh
   ```

4. **Check Code Quality**:
   ```bash
   ./dev-tools/lint-check.sh
   ```

## 📋 Tool Categories

### Essential Daily Tools
- setup-dev-env
- dev-server
- test-runner
- lint-check

### Quality Assurance
- coverage-report
- benchmark
- integration-test
- stress-test

### Release Management
- clean-build
- release-prep
- version-bump
- package-dist

### Debugging & Profiling
- debug-transpiler
- profile-compiler
- validate-examples

## 🔧 Configuration

Tools can be configured via:
- Environment variables
- `.nagari-dev.json` configuration file
- Command-line arguments

See individual tool documentation for specific options.

## 📚 Documentation

Each tool includes:
- Built-in help (`--help` flag)
- Usage examples
- Configuration options
- Error handling

## 🤝 Contributing

When adding new development tools:
1. Follow naming convention: `kebab-case.sh/.bat`
2. Include both Unix and Windows versions
3. Add comprehensive help text
4. Update this README
5. Add tests for the tool itself

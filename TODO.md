# 📋 Nagari Programming Language - TODO List

This document tracks all pending tasks, improvements, and known issues across the Nagari project. Items are organized by priority and component.

## 📍 How to Use This TODO List

Each TODO item includes:
- **Task Description**: What needs to be done
- **File Location**: Exact file path where the TODO is located
- **Line Number**: Specific line number in the source code
- **Original Comment**: The actual TODO comment from the code

**Example Format:**
```
- [ ] Task description
  - `path/to/file.rs:123` - TODO: Original comment from code
```

**Finding TODOs in Code:**
```bash
# Search for all TODO comments
grep -r "TODO\|FIXME\|XXX" src/

# Search with line numbers
grep -rn "TODO" src/
```

## 🔥 High Priority (Critical)

### Core Language Features
- [ ] **Complete WebAssembly Integration** (`src/nagari-wasm/`)
  - [ ] Fix VM bytecode execution support
    - `src/nagari-wasm/src/lib.rs:94` - TODO: The current VM only supports bytecode execution, not direct source code
  - [ ] Implement `eval()` method for direct source code execution
    - `src/nagari-wasm/src/lib.rs:101` - TODO: eval method not available in current VM
  - [ ] Add `call()` method for function invocation
    - `src/nagari-wasm/src/lib.rs:107` - TODO: call method not available in current VM
  - [ ] Implement `load_module()` for module loading
    - `src/nagari-wasm/src/lib.rs:113` - TODO: load_module method not available in current VM
  - [ ] Add `register_function()` for external function binding
    - `src/nagari-wasm/src/lib.rs:139` - TODO: register_function method not available in current VM
  - [ ] Implement performance statistics API
    - `src/nagari-wasm/src/lib.rs:145` - TODO: get_performance_stats method not available in current VM
  - [ ] Add VM reset functionality
    - `src/nagari-wasm/src/lib.rs:152` - TODO: reset method not available in current VM
  - [ ] Fix web_sys navigator API integration
    - `src/nagari-wasm/src/lib.rs:286` - TODO: Fix web_sys navigator API
  - [ ] Fix web_sys local_storage API integration
    - `src/nagari-wasm/src/lib.rs:307` - TODO: Fix web_sys local_storage API
    - `src/nagari-wasm/src/lib.rs:313` - TODO: Fix web_sys local_storage API

- [ ] **Parser Implementation Completion** (`src/nagari-parser/`)
  - [ ] Complete ignored parser tests (currently marked with `#[ignore]`)
    - `src/nagari-parser/src/lib.rs:36` - #[ignore] // TODO: Parser implementation needs to be completed
  - [ ] Add semantic validation to parsing process
    - `src/nagari-parser/src/lib.rs:27` - TODO: Add semantic validation
  - [ ] Support more for loop variants (range, iterables, etc.)
    - `src/nagari-parser/src/parser.rs:298` - TODO: Support more for loop variants
  - [ ] Implement comprehensive syntax tree validation

- [ ] **Embedded Systems Support** (`src/nagari-embedded/`)
  - [ ] Fix VM bytecode execution for embedded platforms
    - `src/nagari-embedded/src/lib.rs:341` - TODO: The current VM only supports bytecode execution, not direct source code
  - [ ] Implement function calling API for embedded use
    - `src/nagari-embedded/src/lib.rs:380` - TODO: The current VM doesn't support function calling API
  - [ ] Add resource constraint management
  - [ ] Optimize memory usage for limited environments

### Language Server Protocol (LSP)
- [ ] **Complete LSP Implementation** (`src/lsp-server/`)
  - [ ] Implement workspace folder management
    - `src/lsp-server/src/backend.rs:342` - TODO: Implement workspace folder management
  - [ ] Add comprehensive code actions support
    - `src/lsp-server/src/code_actions.rs:5` - TODO: Add fields for tracking code actions
    - `src/lsp-server/src/code_actions.rs:14` - TODO: Implement actual code actions
  - [ ] Implement code action resolution
    - `src/lsp-server/src/code_actions.rs:19` - TODO: Implement actual code action resolve
  - [ ] Add refactoring capabilities
  - [ ] Implement symbol renaming
  - [ ] Add go-to-definition for imports and modules

## 🚧 Medium Priority (Important)

### Standard Library Expansion
- [ ] **Enhanced Standard Library** (`stdlib/`)
  - [ ] Complete async/await integration in `http.nag`
  - [ ] Add WebSocket support to `http.nag`
  - [ ] Implement file streaming in `fs.nag`
  - [ ] Add encryption/decryption to `crypto.nag`
  - [ ] Implement database ORM patterns in `db.nag`
  - [ ] Add timezone support to `time.nag`
  - [ ] Create `collections.nag` for advanced data structures
  - [ ] Add `testing.nag` for unit testing framework
  - [ ] Create `logging.nag` for structured logging
  - [ ] Add `regex.nag` for pattern matching

### Language Features
- [ ] **Advanced Comprehensions** (`tests/fixtures/test_comprehensions.nag`)
  - [ ] Implement dictionary comprehensions
    - `tests/fixtures/test_comprehensions.nag:12` - # TODO: Dict comprehension
  - [ ] Add set comprehensions
    - `tests/fixtures/test_comprehensions.nag:16` - # TODO: Set comprehension
  - [ ] Support nested comprehensions
  - [ ] Add conditional comprehensions

- [ ] **Type System Enhancement**
  - [ ] Add generic types support
  - [ ] Implement union types
  - [ ] Add intersection types
  - [ ] Implement type inference improvements
  - [ ] Add runtime type checking options

### Package Management
- [ ] **Registry Server Completion** (`src/registry-server/`)
  - [ ] Replace placeholder JWT secret with secure implementation
    - `src/registry-server/src/auth.rs:64` - TODO: Replace with your JWT secret
  - [ ] Add proper authentication logic
    - `src/registry-server/src/middleware.rs:94` - TODO: Add authentication logic here
  - [ ] Implement database migrations
    - `src/registry-server/src/db.rs:21` - TODO: Run database migrations
    - `src/registry-server/src/db.rs:41` - TODO: Add actual migrations
  - [ ] Add package versioning support
  - [ ] Implement dependency resolution
  - [ ] Add package signing and verification

### Testing Infrastructure
- [ ] **Integration Testing** (`src/cli/tests/`)
  - [ ] Fix disabled integration tests (currently marked with `#[ignore]`)
    - `src/cli/tests/integration_tests.rs:40` - #[ignore] // Integration tests require complex binary path setup
    - `src/cli/tests/integration_tests.rs:67` - #[ignore] // Integration tests require proper binary setup
    - `src/cli/tests/integration_tests.rs:102` - #[ignore] // Integration tests require proper binary setup
    - `src/cli/tests/integration_tests.rs:129` - #[ignore] // Integration tests require proper binary setup
    - `src/cli/tests/integration_tests.rs:165` - #[ignore] // Integration tests require proper binary setup
    - `src/cli/tests/integration_tests.rs:209` - #[ignore] // Integration tests require proper binary setup
    - `src/cli/tests/integration_tests.rs:246` - #[ignore] // Integration tests require proper binary setup
    - `src/cli/tests/integration_tests.rs:285` - #[ignore] // Integration tests require proper binary setup
  - [ ] Implement proper binary path setup for tests
  - [ ] Add end-to-end testing framework
  - [ ] Create performance benchmarks
  - [ ] Add memory usage tests

## 📈 Low Priority (Nice to Have)

### Developer Experience
- [ ] **Enhanced CLI Tools**
  - [ ] Add interactive debugger
  - [ ] Implement code coverage reporting
  - [ ] Add performance profiling tools
  - [ ] Create dependency analyzer
  - [ ] Add code complexity metrics

- [ ] **Documentation Generation**
  - [ ] Auto-generate API documentation from code
  - [ ] Create interactive tutorials
  - [ ] Add code examples for all stdlib functions
  - [ ] Implement doc-tests support

### Ecosystem Integration
- [ ] **Editor Support**
  - [ ] Create VS Code extension
  - [ ] Add Vim/Neovim plugin
  - [ ] Implement Emacs mode
  - [ ] Create Sublime Text package

- [ ] **Build System Enhancements**
  - [ ] Add cross-compilation support
  - [ ] Implement incremental compilation
  - [ ] Add dependency caching
  - [ ] Create reproducible builds

### Advanced Features
- [ ] **Concurrency and Parallelism**
  - [ ] Implement actor model
  - [ ] Add parallel collections
  - [ ] Create worker thread support
  - [ ] Implement async streams

- [ ] **Metaprogramming**
  - [ ] Add macro system
  - [ ] Implement reflection capabilities
  - [ ] Create code generation tools
  - [ ] Add compile-time execution

## 🐛 Known Issues

### WASM Integration Issues
- Current VM only supports bytecode execution, not direct source code
- Missing eval, call, and load_module methods in VM interface
- Web APIs (navigator, localStorage) need proper integration
- Performance statistics not available

### Testing Issues
- Integration tests are disabled due to binary path setup complexity
- Parser tests are incomplete and mostly ignored
- Missing comprehensive test coverage for edge cases

### Documentation Gaps
- API reference needs completion
- Missing troubleshooting guides for specific errors
- Incomplete examples for advanced features

## 🎯 Roadmap Alignment

### Version 0.4.0 Goals
- [ ] Complete WASM integration
- [ ] Fix all ignored tests
- [ ] Implement basic LSP features
- [ ] Expand standard library

### Version 0.5.0 Goals
- [ ] Advanced type system
- [ ] Complete package registry
- [ ] Enhanced IDE support
- [ ] Performance optimizations

### Version 1.0.0 Goals
- [ ] Production-ready stability
- [ ] Complete feature set
- [ ] Comprehensive documentation
- [ ] Ecosystem maturity

## 📝 Contributing Guidelines

When working on TODO items:

1. **Check Dependencies**: Ensure your task doesn't depend on incomplete components
2. **Update Tests**: Add or fix tests for your implementation
3. **Document Changes**: Update relevant documentation
4. **Follow Patterns**: Maintain consistency with existing code style
5. **Performance**: Consider performance implications of changes

## 📊 Progress Tracking

- **High Priority**: 0/15 complete (0%)
  - WASM Integration: 10 specific TODOs identified
  - Parser Implementation: 3 specific TODOs identified
  - Embedded Support: 2 specific TODOs identified
  - LSP Implementation: 3 specific TODOs identified
- **Medium Priority**: 0/20 complete (0%)
  - Registry Server: 4 specific TODOs identified
  - Integration Tests: 8 specific TODOs identified
  - Comprehensions: 2 specific TODOs identified
- **Low Priority**: 0/15 complete (0%)
- **Known Issues**: 8 identified
- **Total Tracked TODOs**: 32 with exact locations
- **Total Project Items**: 58 tracked

### TODO Comment Distribution
- `src/nagari-wasm/`: 10 TODOs
- `src/nagari-parser/`: 3 TODOs
- `src/nagari-embedded/`: 2 TODOs
- `src/lsp-server/`: 3 TODOs
- `src/registry-server/`: 4 TODOs
- `src/cli/tests/`: 8 TODOs
- `tests/fixtures/`: 2 TODOs

---

**Last Updated**: July 20, 2025
**Next Review**: August 1, 2025

## 🔍 Finding Additional TODOs

> 💡 **Commands to find TODO items:**
>
> ```bash
> # Find all TODO comments with line numbers
> grep -rn "TODO\|FIXME\|XXX" src/
>
> # Find ignored tests
> grep -rn "#\[ignore\]" src/
>
> # Find unimplemented functions
> grep -rn "unimplemented!\|todo!" src/
>
> # Find placeholder comments
> grep -rn "placeholder\|stub\|not implemented" src/
> ```
>
> **When adding new TODOs:**
> 1. Use consistent format: `// TODO: Description`
> 2. Include issue number if applicable: `// TODO(#123): Description`
> 3. Update this TODO.md file with the new item
> 4. Include exact file path and line number

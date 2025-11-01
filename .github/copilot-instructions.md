# Nagari Programming Language - AI Coding Agent Guide

## Project Overview
Nagari is a production-ready programming language combining Python's elegant syntax with JavaScript's performance. Built with Rust (compiler/tooling) and TypeScript (runtime), it transpiles `.nag` files to JavaScript for universal compatibility.

**Current Version**: 0.3.0+ (Production Ready)
**Target**: Web development with Python-like syntax
**Key Stack**: Rust (compiler), TypeScript (runtime), Node.js/Bun (execution)

## Critical Architecture Knowledge

### 1. Multi-Component Workspace Structure
This is a **Cargo workspace** with 8 distinct crates in `src/`:
- `cli/` - Main user-facing tool (`nag` binary)
- `nagari-compiler/` - Core transpiler (`nagc` binary)
- `nagari-parser/` - Lexer and AST generation
- `lsp-server/` - Language Server Protocol (`nagari-lsp` binary)
- `nagari-vm/` - Virtual machine (future native execution)
- `nagari-wasm/` - WebAssembly support
- `nagari-embedded/` - Embedded systems support
- `registry-server/` - Package registry

**Runtime is separate**: `nagari-runtime/` (TypeScript/npm package) lives at project root, NOT in `src/`.

### 2. Compilation Pipeline Flow
```
.nag file → Lexer (parser) → AST → Transpiler → .js file
                                         ↓
                              Runtime Helpers Injection
                                         ↓
                              Node.js/Bun Execution
```

**Key Pattern**: The transpiler (`src/nagari-compiler/src/transpiler/mod.rs`) injects runtime helper imports automatically:
- `builtin_map.rs` - Maps Python builtins (range, enumerate) to JS equivalents
- `js_runtime.rs` - Generates polyfills and helper functions
- `modules.rs` - Handles import/from statements with ES6 modules

### 3. Runtime Interop System
The runtime (`nagari-runtime/src/`) provides bidirectional JS↔Nagari conversion:
- `interop.ts` - Type conversion and function wrapping
- `builtins.ts` - Python-like functions (range, enumerate, zip, etc.)
- `types.ts` - Type definitions and utilities

**Critical**: All transpiled code requires `nagari-runtime` installed. The CLI auto-checks/installs it.

## Essential Development Workflows

### Building & Testing (Windows/Linux/macOS)
```bash
# Build all components (from project root)
cargo build --release                    # All Rust binaries
cd nagari-runtime && npm install && npm run build  # Runtime

# Quick test run
cargo run --bin nag -- run examples/hello.nag

# Full test suite
./scripts/tools/test-toolchain.sh       # Comprehensive testing
cargo test --workspace                   # Rust unit tests
cd nagari-runtime && npm test           # Runtime tests
```

### Master Development Tool
**Use `dev-tools/dev.sh` (or `.bat`) as the central command interface**:
```bash
./dev-tools/dev.sh dev      # Setup + start dev server
./dev-tools/dev.sh check    # Lint + test everything
./dev-tools/dev.sh ship     # Full release prep (lint + test + package)
```

This tool orchestrates ALL other scripts. Don't bypass it for common workflows.

### Release Packaging
```bash
# Build standalone packages (includes binaries, runtime, stdlib, examples)
./scripts/package-release.sh 0.3.1 windows   # Single platform
./scripts/package-release.sh 0.3.1 all       # All platforms

# Output: packages/nagari-VERSION-TARGET.{tar.gz|zip}
```

Each package is self-contained with install scripts, no compilation needed by end users.

## Project-Specific Patterns & Conventions

### 1. Error Handling in Transpiler
Uses `Result<T, NagariError>` throughout. **Never panic in compiler code**:
```rust
// Good
self.transpile_statement(stmt)?;

// Bad - Don't do this
let result = self.transpile_statement(stmt).unwrap();
```

### 2. AST Node Naming Convention
Located in `src/nagari-parser/src/ast.rs` and `src/nagari-compiler/src/ast.rs` (two versions):
- **Parser AST**: Raw syntax tree, position tracking for errors
- **Compiler AST**: Type-annotated, ready for code generation

When modifying language features, update BOTH AST definitions.

### 3. Runtime Helper Registration Pattern
New Python-like builtins go in `nagari-runtime/src/builtins.ts`:
```typescript
export function myBuiltin(args: any[]): any {
    // Implementation
}

// Export for InteropRegistry auto-registration
export const BUILTINS = { myBuiltin, range, enumerate, ... };
```

Then transpiler auto-injects: `import { myBuiltin } from 'nagari-runtime';`

### 4. Test Organization Strategy
- `examples/` - Full programs, used for integration testing (23+ examples)
- `samples/` - Simple test snippets, minimal language feature demos
- `test-files/` - Parser/compiler test fixtures
- `tests/` - Rust unit test outputs

**Pattern**: Always add a `.nag` example before implementing a language feature.

### 5. Version Synchronization
**Three version numbers MUST match** when releasing:
1. `Cargo.toml` (workspace.package.version)
2. `nagari-runtime/package.json` (version)
3. Documentation references in `README.md`, `docs/`

Use `./dev-tools/dev.sh version [major|minor|patch]` to auto-sync.

### 6. Cross-Platform Script Duality
Nearly every `.sh` script has a `.bat` equivalent:
- `scripts/build.sh` + `scripts/build.bat`
- `dev-tools/dev.sh` + `dev-tools/dev.bat`

**When modifying scripts**: Update both versions to maintain Windows/Unix parity.

## Critical Integration Points

### 1. CLI → Compiler → Runtime Chain
The `nag` CLI (`src/cli/src/main.rs`) orchestrates:
1. Calls `nagari-compiler` to transpile `.nag` → `.js`
2. Checks `nagari-runtime` installed in `node_modules/`
3. Auto-installs runtime if missing
4. Executes with Node.js or Bun (auto-detected)

**Don't bypass this flow** - direct compiler usage skips runtime checks.

### 2. LSP Server Independence
`lsp-server/` is a standalone binary (`nagari-lsp`) using `tower-lsp`:
- Uses `nagari-parser` for syntax analysis
- Maintains its own document cache (`dashmap` for thread-safety)
- Communicates via JSON-RPC (stdio or TCP)

**Pattern**: LSP changes don't require CLI/compiler rebuilds, but share parser crate.

### 3. Package Manager Integration
`src/cli/src/package/` implements a full package management system:
- `manifest.rs` - `nagari.toml` parsing
- `resolver.rs` - Dependency resolution algorithm
- `registry.rs` - HTTP client for package registry
- `lockfile.rs` - `nagari.lock` deterministic installs

**Future-ready**: Already designed for npm-style package ecosystem.

## Common Pitfalls & Solutions

### 1. "Runtime not found" errors
**Cause**: Transpiled JS imports `nagari-runtime`, but it's not installed.
**Fix**: Always run via `nag run` (not direct `node`), or manually `npm install nagari-runtime`.

### 2. Indentation parsing issues
**Pattern**: Nagari uses Python-style indentation. Lexer (`src/nagari-parser/src/lexer.rs`) tracks INDENT/DEDENT tokens.
**Debug**: Enable `RUST_LOG=debug` to see token stream: `RUST_LOG=debug cargo run -- run file.nag`

### 3. Transpiler output isn't updating
**Cause**: Cached JS output in `dist/` or `target/`.
**Fix**: `./dev-tools/dev.sh clean` or `cargo clean && rm -rf dist/`

### 4. Cross-compilation failures
**Pattern**: Building for non-native targets (e.g., Windows on Linux) requires cross-compilation setup.
**Solution**: Use the packaging scripts which handle target-specific builds: `./scripts/package-release.sh VERSION linux`

### 5. Runtime type conversion errors
**Pattern**: JS values crossing into Nagari must be explicitly converted via `InteropRegistry`.
**Debug**: Check `nagari-runtime/src/interop.ts` for type mappings. Add new converters as needed.

## Documentation & Discovery

### Where to Find Information
- **Language spec**: `specs/language-spec.md` (canonical grammar)
- **API reference**: `docs/api-reference.md` (standard library)
- **Architecture**: `docs/architecture.md` (deep dive on internals)
- **Development guide**: `docs/development-guide.md` (contributor onboarding)
- **Build system**: `docs/PACKAGING.md` (release process)

### Key Example Files
- `examples/algorithms.nag` - Data structures, algorithms
- `examples/react_todo_app.nag` - JSX integration
- `examples/express_server.nag` - HTTP server patterns
- `examples/async_demo.nag` - Async/await usage

**Pattern**: When answering "how do I...?" questions, reference examples/ first.

## Quick Reference Commands
```bash
# Development
./dev-tools/dev.sh dev              # Complete dev setup + server
cargo run --bin nag -- run FILE     # Execute Nagari file
cargo run --bin nagc -- FILE        # Direct transpilation

# Testing
./dev-tools/dev.sh test             # All tests
cargo test -p nagari-compiler       # Specific crate tests
./scripts/tools/test-examples.sh    # Validate all examples

# Building
cargo build --release --bin nag     # CLI tool
cargo build --release --workspace   # All binaries
cd nagari-runtime && npm run build  # Runtime library

# Packaging
./scripts/package-release.sh 0.3.1 current  # Current platform
./scripts/package-release.sh 0.3.1 all      # Multi-platform

# LSP (for editor integration)
cargo run --bin nagari-lsp -- --mode stdio
```

## Maintenance Guidelines
- **Before commits**: Run `./dev-tools/dev.sh check` (lint + test)
- **When adding language features**: Update parser AST, compiler AST, transpiler, AND add example
- **When changing runtime**: Bump version in `package.json` AND `Cargo.toml`
- **When updating dependencies**: Check both Rust (`Cargo.toml`) and npm (`package.json`)

---

## Deep Dive: Key Subsystems

### LSP Server Architecture
The Language Server (`lsp-server/`) uses `tower-lsp` framework with these key components:
- **Backend** (`backend.rs`) - Core LSP protocol implementation using `tower-lsp::LanguageServer` trait
- **Document Management** (`document.rs`) - Thread-safe document cache using `DashMap<Url, String>`
- **Completion Engine** (`completion.rs`) - Context-aware suggestions from AST analysis
- **Diagnostics** (`diagnostics.rs`) - Real-time error detection via parser integration
- **Capabilities** (`capabilities.rs`) - Negotiates LSP features with clients

**Communication Pattern**: Supports both stdio (VS Code) and WebSocket (browser editors):
```rust
// WebSocket messages forwarded to internal LSP pipes
ws_stream → extract_lsp_message() → client_writer → LspService → server_reader → ws_sender
```

**Critical**: LSP maintains separate document state from CLI/compiler. Changes require parser re-parsing.

### Package Manager Design
The package system (`src/cli/src/package/`) implements npm-compatible semantics:
- **Manifest** (`manifest.rs`) - Parses `nagari.toml` with support for:
  - Version specs (semver), path deps, git deps, registry deps
  - Workspace configuration, compile options, runtime options
  - Platform-specific dependencies (`os`, `cpu` fields)
- **Resolver** (`resolver.rs`) - Dependency resolution algorithm handles:
  - Version range satisfaction using semver
  - Circular dependency detection
  - Peer dependency resolution
  - Optional dependency handling
- **Lockfile** (`lockfile.rs`) - Ensures reproducible builds with:
  - Exact resolved versions and sources
  - Integrity checksums (SHA256)
  - Metadata caching for offline installs

**Design Philosophy**: Follow npm conventions for JavaScript ecosystem compatibility. Package names map to `node_modules/` structure.

### Cross-Platform Build System
The build system (`scripts/package-cross-platform.sh`) handles multi-target compilation:
- **Target Matrix**: 5 supported targets (Windows x64, Linux x64/ARM64, macOS Intel/ARM)
- **Build Strategy**:
  1. Runtime built once (TypeScript, target-agnostic)
  2. Per-target Rust compilation with `cargo build --release --target`
  3. Bundle creation with binaries + runtime + stdlib + examples
  4. Platform-specific installers (`.sh` for Unix, `.bat` for Windows)
- **Cross-Compilation**: Uses `rustup target add` for non-native targets
- **Package Structure**: Self-contained with install scripts that:
  - Copy binaries to `~/.nagari/bin/`
  - Install runtime to `~/.nagari/nagari-runtime/`
  - Add PATH instructions for shell config

**Common Issue**: Cross-compiling to macOS requires Xcode SDK. Use native macOS builds or GitHub Actions.

---

## Naming Conventions & Code Organization

### Module Naming Patterns
- **Binary crates**: Short names (`nag`, `nagc`, `nagari-lsp`)
- **Library crates**: Descriptive names (`nagari-compiler`, `nagari-parser`)
- **Module files**: Snake_case (`builtin_map.rs`, `js_runtime.rs`)
- **Types**: PascalCase (`JSTranspiler`, `NagariError`, `PackageManifest`)
- **Functions**: Snake_case (`transpile_statement`, `detect_javascript_runtime`)

### File Organization Pattern
Crates follow consistent structure:
```
crate-name/
├── src/
│   ├── main.rs or lib.rs    # Entry point
│   ├── error.rs              # Error types (if applicable)
│   ├── config.rs             # Configuration (if applicable)
│   ├── module_name.rs        # Single-file modules
│   └── complex_module/       # Multi-file modules
│       ├── mod.rs            # Module coordinator
│       ├── submodule1.rs
│       └── submodule2.rs
├── tests/                    # Integration tests
├── benches/                  # Benchmarks (optional)
└── Cargo.toml
```

### Commit Message Format
Follow Conventional Commits specification:
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`
**Scopes**: Component names (`parser`, `transpiler`, `runtime`, `cli`, `lsp`, `docs`)

**Examples**:
- `feat(parser): add pattern matching support for tuples`
- `fix(transpiler): handle nested async arrow functions correctly`
- `docs(api): update range() function documentation`
- `perf(compiler): optimize AST traversal for large files`
- `refactor(runtime): extract type conversion to separate module`

**Breaking Changes**: Add `BREAKING CHANGE:` footer or `!` after type: `feat(cli)!: remove deprecated --legacy flag`

---

## Known Pain Points & Workarounds

### 1. Parser Indentation Edge Cases
**Issue**: Indentation parsing struggles with mixed tabs/spaces and comments at dedent boundaries.
**Root Cause**: Lexer (`nagari-parser/src/lexer.rs`) tracks indent stack but doesn't normalize whitespace.
**Workaround**: Configure editor to use 4 spaces (never tabs). Avoid inline comments before dedents.
**TODO**: Tracked in `src/nagari-parser/src/lexer.rs` - need whitespace normalization pass.

### 2. Runtime Auto-Install Race Condition
**Issue**: Multiple simultaneous `nag run` invocations can conflict during runtime install.
**Root Cause**: CLI checks `node_modules/nagari-runtime` existence but lacks file locking.
**Workaround**: Run `npm install nagari-runtime` manually before parallel execution.
**Fix Pending**: Add file locking in `src/cli/src/main.rs` runtime check logic.

### 3. Cross-Compilation to macOS
**Issue**: Building macOS packages on Linux/Windows requires Apple SDK.
**Root Cause**: Rust cross-compilation needs system libraries not available outside macOS.
**Workaround**: Use native macOS machine or GitHub Actions runners for macOS targets.
**Alternative**: Package script detects unavailable targets and gracefully skips with warnings.

### 4. LSP Document Synchronization
**Issue**: LSP diagnostics occasionally stale after rapid edits.
**Root Cause**: Document update events queued faster than parser can process.
**Workaround**: LSP debounces parsing by 300ms in `diagnostics.rs`.
**Monitor**: Check `RUST_LOG=debug` output for "Parsing document" messages if diagnostics lag.

### 5. Transpiler Helper Injection Order
**Issue**: Runtime helpers sometimes referenced before import statement.
**Root Cause**: Transpiler adds imports at top but helpers at bottom of output.
**Pattern**: Always check transpiled JS has imports before usage. Look for:
```javascript
import { InteropRegistry, range, enumerate } from 'nagari-runtime';  // Top
// ... user code ...
// Helper functions (if any)  // Bottom
```
**Debug**: Add `--output debug.js` to inspect transpiled output when runtime errors occur.

---

This guide provides the "invisible knowledge" needed to be productive immediately. For detailed API docs, see the docs/ directory. For specific questions about implementation, examine the relevant crate's source code directly.

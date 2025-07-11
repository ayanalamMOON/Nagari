# Architecture Overview

Internal architecture and design principles of the Nagari programming language.

## System Overview

Nagari is a modern programming language designed for JavaScript interoperability with a robust multi-component architecture:

```
┌─────────────────────────────────────────────────┐
│                User Code                        │
│              (.nag files)                       │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│               Parser                            │
│           (nagari-parser)                       │
│        Lexer → AST → Validation                 │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│               Compiler                          │
│          (nagari-compiler)                      │
│     Optimization → Bytecode → Target Code       │
└─────────────────┬───────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────┐
│               Runtime                           │
│         (nagari-runtime)                        │
│      VM → JavaScript Bridge → Execution         │
└─────────────────────────────────────────────────┘
```

## Core Components

### 1. Nagari Parser (`nagari-parser`)

**Purpose:** Lexical analysis and syntax parsing
**Location:** `nagari-parser/src/`
**Language:** Rust

```rust
// High-level parser flow
Source Code → Lexer → Tokens → Parser → AST → Validator → Typed AST
```

**Key Modules:**
- `lexer.rs` - Tokenization and character stream processing
- `parser.rs` - Recursive descent parser for Nagari syntax
- `ast.rs` - Abstract Syntax Tree definitions
- `validator.rs` - Semantic analysis and type checking

**Features:**
- Error recovery and reporting
- Position tracking for debugging
- Support for multiple syntax modes
- Incremental parsing for LSP

### 2. Nagari Compiler (`nagari-compiler`)

**Purpose:** Code generation and optimization
**Location:** `nagari-compiler/src/`
**Language:** Rust

```rust
// Compilation pipeline
Typed AST → Optimizer → Bytecode Generator → Target Emitter → Output
```

**Key Modules:**
- `optimizer.rs` - AST-level optimizations
- `codegen.rs` - Bytecode generation
- `emit/` - Target-specific code emission
  - `js_emitter.rs` - JavaScript output
  - `wasm_emitter.rs` - WebAssembly output
  - `native_emitter.rs` - Native code output

**Optimization Phases:**
1. **Dead Code Elimination** - Remove unused functions/variables
2. **Constant Folding** - Evaluate compile-time constants
3. **Inlining** - Inline small functions for performance
4. **Loop Optimization** - Optimize iteration patterns

### 3. Nagari Runtime (`nagari-runtime`)

**Purpose:** Execution environment and JavaScript bridge
**Location:** `nagari-runtime/src/`
**Language:** TypeScript/JavaScript

```typescript
// Runtime architecture
Bytecode → VM Interpreter → Native Functions → JavaScript APIs
```

**Key Modules:**
- `vm/` - Virtual machine implementation
  - `interpreter.ts` - Bytecode interpreter
  - `memory.ts` - Garbage collection and memory management
  - `stack.ts` - Call stack management
- `bridge/` - JavaScript interoperability
  - `js_bridge.ts` - Function call bridging
  - `type_conversion.ts` - Type mapping between Nagari and JS
- `stdlib/` - Standard library implementation
  - `http.ts` - HTTP client functionality
  - `fs.ts` - File system operations
  - `crypto.ts` - Cryptographic functions

### 4. Virtual Machine (`nagari-vm`)

**Purpose:** Bytecode execution engine
**Location:** `nagari-vm/src/`
**Language:** Rust

```rust
// VM execution cycle
Bytecode → Instruction Fetch → Decode → Execute → Stack Update
```

**Architecture:**
- Stack-based virtual machine
- Register optimization for frequent operations
- Garbage collection integration
- JIT compilation support (planned)

### 5. Language Server (`lsp-server`)

**Purpose:** IDE integration and developer tooling
**Location:** `lsp-server/src/`
**Language:** Rust

**LSP Features:**
- Real-time syntax checking
- Auto-completion
- Go-to-definition
- Symbol search
- Code formatting
- Hover information

### 6. CLI Tools (`cli`)

**Purpose:** Command-line interface and tooling
**Location:** `cli/src/`
**Language:** Rust

**Commands:**
- `run` - Execute Nagari files
- `build` - Compile to target formats
- `repl` - Interactive shell
- `test` - Run test suites
- `format` - Code formatting
- `lsp` - Language server

## Data Flow

### Compilation Flow

```
Source (.nag)
    ↓
┌─────────────┐
│   Lexer     │ → Tokens
└─────────────┘
    ↓
┌─────────────┐
│   Parser    │ → Raw AST
└─────────────┘
    ↓
┌─────────────┐
│  Validator  │ → Typed AST
└─────────────┘
    ↓
┌─────────────┐
│  Optimizer  │ → Optimized AST
└─────────────┘
    ↓
┌─────────────┐
│  Code Gen   │ → Bytecode
└─────────────┘
    ↓
┌─────────────┐
│   Emitter   │ → Target Code (JS/WASM/Native)
└─────────────┘
```

### Runtime Execution Flow

```
Bytecode/JS Code
    ↓
┌─────────────┐
│    VM       │ ← Standard Library
└─────────────┘
    ↓
┌─────────────┐
│ JS Bridge   │ ← Native APIs
└─────────────┘
    ↓
┌─────────────┐
│  Execution  │ → Output
└─────────────┘
```

## Memory Management

### Garbage Collection

Nagari uses a generational garbage collector:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Young Gen   │ → │ Mature Gen  │ → │  Old Gen    │
│ (frequent)  │    │ (periodic)  │    │ (rare)      │
└─────────────┘    └─────────────┘    └─────────────┘
```

**Collection Strategies:**
- **Young Generation:** Copy collection for new objects
- **Mature Generation:** Mark-and-sweep for medium-lived objects
- **Old Generation:** Mark-compact for long-lived objects

### Memory Layout

```
Stack Frame Layout:
┌─────────────┐ ← Frame Pointer (FP)
│ Return Addr │
├─────────────┤
│ Saved FP    │
├─────────────┤
│ Local Vars  │
├─────────────┤
│ Temp Values │
└─────────────┘ ← Stack Pointer (SP)
```

## Type System

### Type Hierarchy

```
Any
├── Undefined
├── Null
├── Boolean
├── Number
│   ├── Integer
│   └── Float
├── String
├── Object
│   ├── Array
│   ├── Function
│   └── Class Instance
└── Symbol
```

### Type Inference

Nagari uses Hindley-Milner type inference with extensions:

```rust
// Type inference algorithm
fn infer_type(expr: &Expr, context: &TypeContext) -> Type {
    match expr {
        Expr::Literal(lit) => infer_literal_type(lit),
        Expr::Variable(name) => context.lookup(name),
        Expr::Call(func, args) => {
            let func_type = infer_type(func, context);
            unify_call(func_type, args, context)
        }
        // ... other expressions
    }
}
```

## Interoperability

### JavaScript Bridge

```typescript
// Type conversion between Nagari and JavaScript
class TypeBridge {
  nagariToJS(value: NagariValue): any {
    switch (value.type) {
      case 'string': return value.data;
      case 'number': return value.data;
      case 'array': return value.data.map(this.nagariToJS);
      case 'object': return this.convertObject(value.data);
      case 'function': return this.wrapFunction(value.data);
    }
  }

  jsToNagari(value: any): NagariValue {
    if (typeof value === 'string') return { type: 'string', data: value };
    if (typeof value === 'number') return { type: 'number', data: value };
    if (Array.isArray(value)) return { type: 'array', data: value.map(this.jsToNagari) };
    // ... other conversions
  }
}
```

### Native Module Integration

```rust
// Native module registration
pub struct NativeModule {
    pub name: String,
    pub functions: HashMap<String, NativeFunction>,
}

impl NativeModule {
    pub fn register_function(&mut self, name: &str, func: NativeFunction) {
        self.functions.insert(name.to_string(), func);
    }
}
```

## Performance Considerations

### Optimization Strategies

1. **Compile-Time Optimizations:**
   - Constant folding and propagation
   - Dead code elimination
   - Function inlining
   - Loop unrolling

2. **Runtime Optimizations:**
   - Inline caching for property access
   - Hidden classes for object layout
   - Speculative optimization
   - Profile-guided optimization

3. **Memory Optimizations:**
   - Object pooling for frequent allocations
   - Packed arrays for homogeneous data
   - String interning for immutable strings
   - Weak references for caches

### Benchmarking

```rust
// Performance measurement framework
pub struct Benchmark {
    name: String,
    setup: fn(),
    test: fn() -> Duration,
    teardown: fn(),
}

impl Benchmark {
    pub fn run(&self, iterations: usize) -> BenchmarkResult {
        (self.setup)();
        let start = Instant::now();
        for _ in 0..iterations {
            (self.test)();
        }
        let duration = start.elapsed();
        (self.teardown)();

        BenchmarkResult {
            name: self.name.clone(),
            iterations,
            total_time: duration,
            avg_time: duration / iterations as u32,
        }
    }
}
```

## Security Model

### Sandboxing

```rust
// Security context for execution
pub struct SecurityContext {
    pub allowed_apis: HashSet<String>,
    pub file_permissions: FilePermissions,
    pub network_permissions: NetworkPermissions,
    pub resource_limits: ResourceLimits,
}

impl SecurityContext {
    pub fn check_api_access(&self, api: &str) -> bool {
        self.allowed_apis.contains(api)
    }

    pub fn check_file_access(&self, path: &Path, mode: AccessMode) -> bool {
        self.file_permissions.allows(path, mode)
    }
}
```

### Input Validation

All external inputs are validated at boundaries:
- Source code parsing with bounds checking
- Runtime type checking for foreign function calls
- Memory bounds checking for array access
- Path validation for file operations

## Extension Points

### Custom Backends

```rust
// Backend trait for code generation
pub trait CodegenBackend {
    fn emit_function(&mut self, func: &Function) -> Result<(), CodegenError>;
    fn emit_expression(&mut self, expr: &Expression) -> Result<(), CodegenError>;
    fn finalize(&mut self) -> Result<Vec<u8>, CodegenError>;
}

// JavaScript backend implementation
pub struct JavaScriptBackend {
    output: String,
    indent_level: usize,
}

impl CodegenBackend for JavaScriptBackend {
    fn emit_function(&mut self, func: &Function) -> Result<(), CodegenError> {
        writeln!(self.output, "function {}(", func.name)?;
        // ... emit function body
        Ok(())
    }
}
```

### Plugin System

```rust
// Plugin interface
pub trait Plugin {
    fn name(&self) -> &str;
    fn initialize(&mut self, context: &mut PluginContext) -> Result<(), PluginError>;
    fn on_parse(&mut self, ast: &mut AST) -> Result<(), PluginError>;
    fn on_compile(&mut self, bytecode: &mut Bytecode) -> Result<(), PluginError>;
}

// Plugin manager
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn load_plugin<P: Plugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }

    pub fn run_parse_hooks(&mut self, ast: &mut AST) -> Result<(), PluginError> {
        for plugin in &mut self.plugins {
            plugin.on_parse(ast)?;
        }
        Ok(())
    }
}
```

## Testing Architecture

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_basic_expression() {
        let source = "2 + 3 * 4";
        let tokens = lexer::tokenize(source).unwrap();
        let ast = parser::parse(tokens).unwrap();

        assert_eq!(ast.kind, ASTKind::BinaryOp);
        assert_eq!(ast.left.unwrap().kind, ASTKind::Number(2));
    }
}
```

### Integration Testing

```rust
#[test]
fn test_end_to_end_compilation() {
    let source = r#"
        function fibonacci(n) {
            if (n <= 1) return n;
            return fibonacci(n-1) + fibonacci(n-2);
        }
        console.log(fibonacci(10));
    "#;

    let result = compile_and_run(source).unwrap();
    assert_eq!(result.stdout, "55\n");
}
```

## Build System

### Cargo Workspace

```toml
[workspace]
members = [
    "nagari-parser",
    "nagari-compiler",
    "nagari-vm",
    "nagari-runtime",
    "lsp-server",
    "cli"
]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

### Cross-Compilation

```bash
# Build for multiple targets
cargo build --target x86_64-unknown-linux-gnu
cargo build --target x86_64-pc-windows-gnu
cargo build --target x86_64-apple-darwin
cargo build --target wasm32-unknown-unknown
```

## Development Workflow

### Hot Reloading

```rust
// File watcher for development
pub struct HotReloader {
    watcher: RecommendedWatcher,
    reload_tx: mpsc::Sender<PathBuf>,
}

impl HotReloader {
    pub fn watch_directory(&mut self, path: &Path) -> Result<(), Error> {
        self.watcher.watch(path, RecursiveMode::Recursive)?;
        Ok(())
    }

    pub fn on_change<F>(&mut self, callback: F)
    where F: Fn(&Path) + Send + 'static {
        // Handle file change events
    }
}
```

### Debugging Support

```rust
// Debug information generation
pub struct DebugInfo {
    pub source_map: SourceMap,
    pub line_table: LineTable,
    pub variable_info: HashMap<String, VariableDebugInfo>,
}

impl DebugInfo {
    pub fn lookup_position(&self, pc: usize) -> Option<SourcePosition> {
        self.line_table.get(pc)
    }

    pub fn get_variable_value(&self, name: &str, frame: &Frame) -> Option<Value> {
        self.variable_info.get(name)
            .and_then(|info| frame.get_local(info.slot))
    }
}
```

## Future Architecture

### Planned Enhancements

1. **JIT Compilation:**
   - LLVM backend for native code generation
   - Profile-guided optimization
   - Adaptive compilation based on runtime feedback

2. **Distributed Execution:**
   - Actor model for concurrent programming
   - Message passing between isolates
   - Distributed garbage collection

3. **Advanced Type System:**
   - Dependent types for compile-time guarantees
   - Effect system for side effect tracking
   - Ownership system inspired by Rust

4. **Tooling Improvements:**
   - Visual debugger with time-travel debugging
   - Performance profiler with flame graphs
   - Memory usage analyzer

## Contributing to Architecture

### Adding New Components

1. **Create Component Crate:**
   ```bash
   cargo new --lib nagari-newfeature
   ```

2. **Add to Workspace:**
   ```toml
   [workspace]
   members = ["nagari-newfeature"]
   ```

3. **Define Public API:**
   ```rust
   // nagari-newfeature/src/lib.rs
   pub mod api;
   pub use api::*;
   ```

4. **Integrate with CLI:**
   ```rust
   // cli/src/main.rs
   use nagari_newfeature::NewFeatureCommand;
   ```

### Architecture Guidelines

- **Separation of Concerns:** Each component has a single responsibility
- **Interface Design:** Use traits for abstraction and testability
- **Error Handling:** Consistent error types across components
- **Performance:** Profile before optimizing, measure impact
- **Documentation:** Every public API needs documentation

---

*Understanding the architecture helps you contribute effectively to Nagari's development.*

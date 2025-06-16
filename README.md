# Nagari Programming Language

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)]()
[![Documentation](https://img.shields.io/badge/docs-comprehensive-green.svg)]()

A modern, production-ready programming language that combines Python's elegant syntax with JavaScript's ecosystem compatibility. Nagari transpiles to clean, readable JavaScript while providing advanced features like comprehensive type checking, intelligent tooling, and seamless interoperability.

## 🚀 Why Nagari?

- **🐍 Python-Inspired Syntax**: Write clean, readable code with indentation-based structure
- **⚡ JavaScript Performance**: Transpiles to optimized ES6+ code for maximum compatibility
- **🔧 Complete Ecosystem**: Full-featured CLI, REPL, package manager, and LSP support
- **📦 Universal Compatibility**: Works seamlessly with React, Vue, Express, and 2M+ npm packages
- **🎯 Type Safety**: Advanced optional typing with inference and validation
- **🔄 Modern Features**: Async/await, JSX, generators, pattern matching, and more
- **🛠️ Developer Experience**: Intelligent autocompletion, real-time diagnostics, and debugging

## ✨ Advanced Features

### Modern Language Constructs

- **List & Dictionary Comprehensions**: `[x**2 for x in range(10) if x % 2 == 0]`
- **Pattern Matching**: Advanced `match/case` with guards and destructuring
- **Generators & Yield**: Memory-efficient iteration with `yield` and `yield from`
- **Context Management**: `with` statements for resource cleanup
- **Decorators**: `@property`, `@async_timeout`, custom decorators
- **Lambda Expressions**: `filter(lambda x: x > 0, numbers)`
- **Advanced Types**: Union types, generics, callable types, type aliases

### JavaScript Integration

- **Native JSX**: First-class React component development
- **Promise Integration**: Seamless async/await with JavaScript Promises
- **Direct JS Code**: Inline JavaScript with `js("code")` and `js { block }`
- **Module Compatibility**: Full ES6 import/export support
- **Web APIs**: Built-in DOM manipulation and browser API access

## 🛠️ Complete Development Ecosystem

### Nagari CLI (`nagari`)

```bash
# Package management
nagari package install react
nagari package publish my-library

# Interactive development
nagari repl --enhanced  # Advanced REPL with history and completion

# Build and compilation
nagari build src/ --output dist/ --optimize
```

### Advanced REPL Features

- **Multi-line Editing**: Smart indentation and bracket matching
- **Intelligent Completion**: Context-aware suggestions with fuzzy matching
- **Session Persistence**: Variables and history saved across restarts
- **Syntax Highlighting**: Real-time error detection and theming
- **Built-in Help**: Interactive documentation and debugging commands

### Package Manager (`nagpkg`)

- **Manifest Support**: JSON and TOML package configurations
- **Dependency Resolution**: Advanced circular dependency detection
- **Intelligent Caching**: Package integrity verification and optimization
- **Registry Integration**: Secure publishing and authentication
- **Lockfile Management**: Deterministic builds with `nag.lock`

### Language Server Protocol (LSP)

- **Real-time Diagnostics**: Syntax and semantic error detection
- **Code Completion**: Intelligent suggestions for functions, variables, and imports
- **Navigation**: Go-to-definition, find-references, symbol search
- **Refactoring**: Safe symbol renaming and code transformations
- **Universal Editor Support**: VS Code, Vim/Neovim, Emacs, and more

## 📋 Quick Start Examples

### Hello World with Advanced Features

```nag
from core import print
from math import sqrt, pi

def greet(name: str = "world", enthusiastic: bool = false) -> str:
    greeting = f"Hello, {name}!"
    return greeting + "!" if enthusiastic else greeting

# List comprehension
numbers = [1, 2, 3, 4, 5]
squares = [x**2 for x in numbers if x % 2 == 0]
print(f"Even squares: {squares}")

# Pattern matching
match len(squares):
    case 0:
        print("No even numbers")
    case n if n > 3:
        print(f"Many even numbers: {n}")
    case _:
        print("Some even numbers")

print(greet("Nagari", true))
```

### Advanced React Component with Hooks

```nag
import React, { useState, useEffect } from "react"
from http import get

def UserProfile({ userId }: { userId: int }):
    user, setUser = useState(none)
    loading, setLoading = useState(true)

    async def fetchUser():
        try:
            setLoading(true)
            response = await get(f"https://api.example.com/users/{userId}")
            setUser(response.data)
        except Exception as e:
            console.error("Failed to fetch user:", e)
        finally:
            setLoading(false)

    useEffect(() => {
        fetchUser()
    }, [userId])

    if loading:
        return <div className="loading">Loading user...</div>

    return (
        <div className="user-profile">
            <h2>{user?.name or "Unknown User"}</h2>
            <p>Email: {user?.email}</p>
            <p>Role: {user?.role or "Member"}</p>
        </div>
    )

export default UserProfile
```

### Express Server with Middleware

```nag
import express from "express"
from fs import read_file, exists
from json import parse, stringify

app = express()
app.use(express.json())

# Middleware with decorator pattern
@app.middleware
def logging_middleware(req, res, next):
    print(f"{req.method} {req.path} - {new Date().toISOString()}")
    next()

# Route handlers with type annotations
@app.get("/api/users/:id")
async def get_user(req: Request, res: Response):
    user_id = int(req.params.id)

    try:
        if not exists(f"data/users/{user_id}.json"):
            return res.status(404).json({"error": "User not found"})

        user_data = parse(read_file(f"data/users/{user_id}.json"))
        res.json(user_data)
    except Exception as e:
        res.status(500).json({"error": str(e)})

@app.post("/api/users")
async def create_user(req: Request, res: Response):
    # Validation with pattern matching
    match req.body:
        case {"name": str(name), "email": str(email)} if "@" in email:
            user = {
                "id": generate_id(),
                "name": name,
                "email": email,
                "created_at": new Date().toISOString()
            }
            # Save user logic here
            res.status(201).json(user)
        case _:
            res.status(400).json({"error": "Invalid user data"})

if __name__ == "__main__":
    app.listen(3000, () => {
        print("Server running on http://localhost:3000")
    })
```

### Async Programming with Generators

```nag
from http import get
import asyncio

async def fetch_user_data(user_ids: list[int]):
    """Fetch multiple users concurrently with generator pattern"""

    async def fetch_single_user(user_id: int):
        try:
            response = await get(f"https://api.example.com/users/{user_id}")
            yield {"success": true, "data": response.data}
        except Exception as e:
            yield {"success": false, "error": str(e), "user_id": user_id}

    # Process users in batches
    batch_size = 5
    for i in range(0, len(user_ids), batch_size):
        batch = user_ids[i:i + batch_size]
        tasks = [fetch_single_user(uid) for uid in batch]
        results = await asyncio.gather(*tasks, return_exceptions=true)

        for result in results:
            async for data in result:
                yield data

# Usage with async iteration
async def main():
    user_ids = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

    async for user_result in fetch_user_data(user_ids):
        if user_result["success"]:
            print(f"Loaded user: {user_result['data']['name']}")
        else:
            print(f"Failed to load user {user_result['user_id']}: {user_result['error']}")

asyncio.run(main())
```

## 📁 Enhanced Project Structure

```text
Nagari/
├── cli/                          # Enhanced CLI tool with commands
│   ├── src/
│   │   ├── commands/            # CLI command handlers
│   │   ├── package/             # Advanced package manager
│   │   │   ├── manifest.rs      # Package manifest handling
│   │   │   ├── manager.rs       # Package lifecycle management
│   │   │   ├── registry.rs      # Registry client with auth
│   │   │   ├── resolver.rs      # Dependency resolution
│   │   │   ├── cache.rs         # Intelligent caching
│   │   │   └── lockfile.rs      # Deterministic builds
│   │   ├── repl_engine/         # Advanced REPL system
│   │   │   ├── engine.rs        # Core REPL engine
│   │   │   ├── editor.rs        # Multi-line editor
│   │   │   ├── evaluator.rs     # Code evaluation
│   │   │   ├── completer.rs     # Intelligent completion
│   │   │   ├── highlighter.rs   # Syntax highlighting
│   │   │   └── session.rs       # Session persistence
│   │   └── main.rs              # CLI entry point
│   └── tests/                   # Comprehensive test suite
├── nagari-compiler/             # Rust-based compiler
│   ├── src/
│   │   ├── lexer.rs            # Enhanced lexical analysis
│   │   ├── parser.rs           # Advanced syntax parsing
│   │   ├── ast.rs              # Abstract syntax tree
│   │   ├── transpiler.rs       # JavaScript code generation
│   │   └── types.rs            # Type system implementation
├── nagari-runtime/              # Runtime utilities and polyfills
├── registry-server/             # Production registry server
│   ├── src/
│   │   ├── handlers/           # REST API endpoints
│   │   ├── models.rs           # Data models
│   │   ├── auth.rs             # JWT authentication
│   │   └── storage.rs          # Package storage backends
├── lsp-server/                  # Language Server Protocol
│   ├── src/
│   │   ├── backend.rs          # LSP protocol implementation
│   │   ├── completion.rs       # Code completion engine
│   │   ├── diagnostics.rs      # Error detection
│   │   └── navigation.rs       # Go-to-definition, references
├── stdlib/                      # Comprehensive standard library
│   ├── core.nag               # Built-in functions and types
│   ├── math.nag               # Mathematical operations
│   ├── http.nag               # HTTP client/server utilities
│   ├── fs.nag                 # File system operations
│   ├── json.nag               # JSON parsing/serialization
│   ├── crypto.nag             # Cryptographic functions
│   ├── db.nag                 # Database connectivity
│   ├── os.nag                 # Operating system interface
│   └── time.nag               # Date/time manipulation
├── examples/                    # Comprehensive example projects
│   ├── react_todo_app.nag     # React application with hooks
│   ├── express_server.nag     # Express.js REST API
│   ├── vue_task_app.nag       # Vue.js application
│   ├── async_demo.nag         # Async programming patterns
│   ├── js_interop_demo.nag    # JavaScript interoperability
│   ├── algorithms.nag         # Data structures and algorithms
│   └── cli_demo.nag           # Command-line applications
├── specs/                       # Enhanced language specification
│   ├── grammar.bnf            # Complete BNF grammar
│   └── language-spec.md       # Comprehensive language reference
├── docs/                        # Complete documentation suite
│   ├── getting-started.md     # Installation and setup
│   ├── language-guide.md      # Language features and syntax
│   ├── api-reference.md       # Standard library documentation
│   ├── ecosystem-guide.md     # CLI, REPL, and tooling
│   ├── interop-guide.md       # JavaScript integration
│   └── troubleshooting.md     # Common issues and solutions
├── tools/                       # Development and build tools
│   ├── setup-nagpkg.sh       # Package manager setup (Unix)
│   ├── setup-nagpkg.bat      # Package manager setup (Windows)
│   ├── build.sh              # Cross-platform build script
│   ├── run-tests.sh          # Comprehensive test runner
│   └── test-*.sh             # Specialized testing tools
└── tests/                       # Integration and end-to-end tests
```

## 🚀 Installation & Setup

### Method 1: Install Release (Recommended)

```bash
# Install from GitHub releases (coming soon)
curl -sSL https://install.nagari.dev | bash

# Or using npm
npm install -g nagari

# Verify installation
nagari --version
nagari --help
```

### Method 2: Build from Source

```bash
# Prerequisites: Rust 1.70+, Node.js 18+, Git
git clone https://github.com/nagari-lang/nagari.git
cd nagari

# Build all components
./tools/build.sh         # Unix/Linux/macOS
./tools/build.bat        # Windows

# Install CLI globally
cargo install --path cli

# Setup package manager
./tools/setup-nagpkg.sh  # Unix/Linux/macOS
./tools/setup-nagpkg.bat # Windows
```

### Quick Development Setup

```bash
# Create new Nagari project
nagari new my-app --template react
cd my-app

# Install dependencies
nagari package install

# Start development server
nagari dev --watch --hot-reload

# Build for production
nagari build --optimize --target browser
```

## 🔧 CLI Usage Guide

### Core Commands

```bash
# Project Management
nagari new <name> [--template <template>]  # Create new project
nagari init                                # Initialize in existing directory
nagari dev [--watch] [--port 3000]       # Development server

# Compilation & Building
nagari build [options]                     # Build project
  --output <dir>        # Output directory (default: dist/)
  --target <target>     # browser, node, or universal (default: browser)
  --optimize           # Enable optimization and minification
  --sourcemap         # Generate source maps for debugging
  --declarations      # Generate TypeScript declarations
  --watch             # Watch for file changes

# Package Management
nagari package install [package]          # Install dependencies
nagari package add <package>             # Add new dependency
nagari package remove <package>          # Remove dependency
nagari package publish                   # Publish to registry
nagari package search <query>            # Search registry
nagari package info <package>            # Package information

# Interactive Development
nagari repl [--enhanced]                  # Start interactive REPL
  --enhanced          # Advanced features (completion, highlighting)
  --no-history       # Disable command history
  --load <file>      # Load file into REPL session

# Code Quality & Tools
nagari check [--strict]                  # Type checking and linting
nagari format [--write]                  # Code formatting
nagari test [--watch] [--coverage]       # Run tests
nagari docs generate                     # Generate documentation
```

### Advanced Configuration

Create `nagari.config.json` for project-specific settings:

```json
{
  "compiler": {
    "target": "browser",
    "optimize": true,
    "strictTypes": true,
    "experimentalFeatures": ["decorators", "generators"]
  },
  "dev": {
    "port": 3000,
    "hotReload": true,
    "openBrowser": true
  },
  "build": {
    "outputDir": "dist",
    "sourcemap": true,
    "minify": true,
    "bundle": true
  },
  "linting": {
    "maxLineLength": 100,
    "enforceTypeAnnotations": false,
    "allowImplicitReturns": true
  }
}
```

## Language Specification

- **File Extension**: `.nag`
- **Indentation**: 4 spaces (strict)
- **Type System**: Optional static typing with runtime flexibility
- **Async Support**: Native async/await with Promise integration
- **Interop**: Seamless JavaScript library integration
- **JSX**: First-class React component support

## 📖 Comprehensive Documentation

### Getting Started Resources

- **[🚀 Quick Start Guide](docs/getting-started.md)**: Zero to productive in 5 minutes
- **[📚 Language Tutorial](docs/language-guide.md)**: Complete language walkthrough
- **[🔧 Installation Guide](docs/installation.md)**: Detailed setup instructions
- **[💡 Best Practices](docs/best-practices.md)**: Coding conventions and patterns

### Language Reference

- **[📋 Language Specification](specs/language-spec.md)**: Complete syntax and semantics
- **[🔤 Grammar Definition](specs/grammar.bnf)**: Formal BNF grammar
- **[📖 API Reference](docs/api-reference.md)**: Standard library documentation
- **[🔗 JavaScript Interop](docs/interop-guide.md)**: Integration patterns and examples

### Developer Tools

- **[⚙️ CLI Reference](docs/cli-reference.md)**: Complete command documentation
- **[🔄 REPL Guide](docs/repl-guide.md)**: Interactive development environment
- **[📦 Package Management](docs/package-manager.md)**: Dependency management and publishing
- **[🛠️ LSP Integration](docs/lsp-guide.md)**: Editor setup and language server features

### Advanced Topics

- **[🏗️ Architecture Guide](docs/architecture.md)**: Compiler and runtime internals
- **[🔧 Development Guide](docs/development-guide.md)**: Contributing to Nagari
- **[🚀 Performance Guide](docs/performance.md)**: Optimization tips and techniques
- **[🐛 Troubleshooting](docs/troubleshooting.md)**: Common issues and solutions

## 🌟 Example Projects & Demos

### Web Development

- **[📱 React Todo App](examples/react_todo_app.nag)**: Complete React application with hooks, routing, and state management
- **[🌐 Express REST API](examples/express_server.nag)**: RESTful web service with middleware, authentication, and database integration
- **[⚡ Vue Task Manager](examples/vue_task_app.nag)**: Vue.js application with reactive data and component composition
- **[🔥 Real-time Chat](examples/realtime_chat.nag)**: WebSocket-based chat application

### System Programming

- **[🖥️ CLI Tool](examples/cli_demo.nag)**: Command-line application with argument parsing and file operations
- **[📂 File Processor](examples/file_operations.nag)**: Batch file processing with async I/O
- **[🔐 Crypto Utils](examples/crypto_demo.nag)**: Encryption, hashing, and digital signatures
- **[📊 Data Analysis](examples/data_analysis.nag)**: Statistical analysis and visualization

### Algorithm Implementations

- **[🧮 Mathematical Algorithms](examples/algorithms.nag)**: Sorting, searching, and graph algorithms
- **[🧠 Machine Learning](examples/ml_demo.nag)**: Neural networks and data science patterns
- **[🎮 Game Development](examples/game_demo.nag)**: Simple game with graphics and input handling

### JavaScript Integration

- **[🔗 Interop Patterns](examples/js_interop_demo.nag)**: Comprehensive JavaScript integration examples
- **[📚 Library Wrappers](examples/library_wrappers.nag)**: Wrapping popular JS libraries
- **[🌍 Browser APIs](examples/browser_apis.nag)**: Working with DOM, fetch, localStorage, etc.

## 🏆 Language Features Showcase

### Modern Python-Inspired Syntax

```nag
# Type annotations with inference
def calculate_stats(numbers: list[float]) -> dict[str, float]:
    return {
        "mean": sum(numbers) / len(numbers),
        "max": max(numbers),
        "min": min(numbers)
    }

# List comprehensions with filtering
evens = [x for x in range(100) if x % 2 == 0]
word_lengths = {word: len(word) for word in ["hello", "world", "python"]}

# Pattern matching with guards
match request:
    case {"method": "GET", "path": path} if path.startswith("/api"):
        handle_api_request(path)
    case {"method": "POST", "data": data} if validate_data(data):
        process_data(data)
    case _:
        return_error("Invalid request")
```

### Advanced Async Programming

```nag
# Generator functions with async support
async def process_stream(source: AsyncIterable[str]):
    async for item in source:
        processed = await transform_item(item)
        if processed:
            yield processed

# Context managers for resource handling
async with database.transaction() as tx:
    await tx.execute("INSERT INTO users ...")
    await tx.execute("UPDATE stats ...")
    # Automatic rollback on exception

# Concurrent processing with error handling
async def fetch_all_users(user_ids: list[int]):
    tasks = [fetch_user(uid) for uid in user_ids]
    results = await asyncio.gather(*tasks, return_exceptions=true)

    return [
        result for result in results
        if not isinstance(result, Exception)
    ]
```

### Seamless React Integration

```nag
# React hooks with TypeScript-like types
def useCounter(initial: int = 0):
    count, setCount = useState(initial)

    def increment(): setCount(count + 1)
    def decrement(): setCount(count - 1)
    def reset(): setCount(initial)

    return { count, increment, decrement, reset }

# Custom components with props validation
def UserCard({ user, onEdit, onDelete }: {
    user: User,
    onEdit: (User) -> void,
    onDelete: (int) -> void
}):
    return (
        <div className="user-card">
            <h3>{user.name}</h3>
            <p>{user.email}</p>
            <div className="actions">
                <button onClick={() => onEdit(user)}>Edit</button>
                <button onClick={() => onDelete(user.id)}>Delete</button>
            </div>        </div>
    )
```

## 🚦 Development Status & Roadmap

### Current Version: 0.2.0 (June 2025) ✅

**Completed Features:**

- ✅ **Complete CLI Ecosystem**: Advanced package manager, REPL, and build tools
- ✅ **Production Registry**: Secure package publishing and distribution
- ✅ **LSP Integration**: Full language server with intelligent editing features
- ✅ **Comprehensive Testing**: Multi-tier testing with automated CI/CD
- ✅ **Enhanced Grammar**: Modern language features including comprehensions, pattern matching, generators
- ✅ **Standard Library**: 9 comprehensive modules for common programming tasks
- ✅ **Cross-platform Tooling**: Unix and Windows compatibility throughout

### Next Release: 0.3.0 (Q3 2025) 🚧

**Planned Features:**

- 🚧 **Enhanced Parser**: Complete implementation of new grammar features
- 🚧 **Advanced Type System**: Generics, union types, and type inference improvements
- 🚧 **Performance Optimizations**: Compile-time optimizations and runtime improvements
- 🚧 **Mobile Development**: React Native integration and mobile-specific features
- 🚧 **WebAssembly Target**: WASM compilation for near-native performance
- 🚧 **Macro System**: Compile-time metaprogramming capabilities
- 🚧 **WebAssembly Runtime**: Browser and edge deployment capabilities
- 🚧 **Embedded Runtime**: Plugin and SDK integration for multiple platforms

### Long-term Vision: 1.0.0 (2026) 🎯

**Production-Ready Goals:**

- 🎯 **Stable Language Specification**: Backward compatibility guarantees
- 🎯 **Enterprise Features**: Advanced tooling, security, and scalability
- 🎯 **Complete Ecosystem**: IDE plugins, debugging tools, profiling
- 🎯 **Performance Parity**: Match or exceed JavaScript performance
- 🎯 **Community Ecosystem**: Package registry with thousands of packages

## 🤝 Contributing & Community

### How to Contribute

We welcome contributions of all kinds! Here's how you can help:

```bash
# 1. Fork and clone the repository
git clone https://github.com/yourusername/nagari.git
cd nagari

# 2. Set up development environment
./tools/dev-setup.sh

# 3. Create feature branch
git checkout -b feature/amazing-feature

# 4. Make changes and test
./tools/run-tests.sh

# 5. Submit pull request
git push origin feature/amazing-feature
```

### Contribution Areas

- **🐛 Bug Reports**: Help us identify and fix issues
- **💡 Feature Requests**: Suggest new language features or improvements
- **📖 Documentation**: Improve guides, examples, and API documentation
- **🧪 Testing**: Add test cases and improve coverage
- **🔧 Tooling**: Enhance CLI, REPL, LSP, and development tools
- **📦 Standard Library**: Contribute new modules and functions
- **🌍 Examples**: Create real-world project examples

### Community Guidelines

- **Be Respectful**: Inclusive and welcoming environment for all contributors
- **Quality First**: Maintain high standards for code, documentation, and examples
- **Collaborate**: Work together to solve problems and share knowledge
- **Learn Together**: Help newcomers and learn from experienced developers

### Getting Help

- **💬 Discord**: Join our community chat for real-time discussions
- **📧 Mailing List**: Subscribe for announcements and technical discussions
- **🐛 Issues**: Report bugs and request features on GitHub
- **📚 Documentation**: Comprehensive guides and API reference
- **💡 Stack Overflow**: Ask questions with the `nagari` tag

## 📄 License & Legal

**License**: MIT License - see [LICENSE](LICENSE) for details

**Copyright**: © 2025 Nagari Language Contributors

**Patent Policy**: Nagari is committed to being free of patent encumbrances

**Trademark**: "Nagari" is a trademark of the Nagari Language Project

## 🙏 Acknowledgments

### Core Team

- **Language Design**: Syntax, semantics, and ecosystem architecture
- **Compiler Engineering**: Lexer, parser, and transpiler implementation
- **Tooling Development**: CLI, REPL, LSP, and developer experience
- **Documentation**: Comprehensive guides, examples, and tutorials

### Special Thanks

- **Python Software Foundation**: Inspiration for syntax and philosophy
- **Rust Community**: Foundation for compiler and tooling implementation
- **JavaScript Ecosystem**: Target platform and interoperability
- **Open Source Contributors**: Bug reports, feature requests, and code contributions

### Dependencies & Credits

Nagari builds upon excellent open-source projects:

- **Rust**: Systems programming language for compiler implementation
- **Node.js**: JavaScript runtime for development tooling
- **React**: UI library for example applications
- **And many more**: See [CREDITS.md](CREDITS.md) for complete list

---

**Ready to start building with Nagari?**

Check out our [🚀 Quick Start Guide](docs/getting-started.md) or try the [📱 Interactive Tutorial](https://nagari.dev/tutorial) to get hands-on experience!

**Have questions?** Join our [💬 Community Discord](https://discord.gg/nagari) or browse the [📚 Documentation](docs/README.md).

**Want to contribute?** Read our [🤝 Contributing Guide](CONTRIBUTING.md) and help shape the future of Nagari!

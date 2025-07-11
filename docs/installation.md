# Installation Guide

Complete installation instructions for the Nagari programming language ecosystem.

## Quick Installation

### 1. Install the Runtime (Node.js/NPM)

```bash
# Install the Nagari runtime from npm
npm install -g nagari-runtime

# Or add to your project
npm install nagari-runtime
```

### 2. Build from Source (Rust)

```bash
# Clone the repository
git clone https://github.com/ayanalamMOON/Nagari.git
cd Nagari

# Build the complete toolchain
cargo build --release

# Run tests to verify installation
cargo test
```

## System Requirements

### Minimum Requirements
- **Node.js**: v16.0.0 or higher
- **Rust**: v1.70.0 or higher (for building from source)
- **Operating System**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 18.04+)
- **Memory**: 2GB RAM minimum, 4GB recommended
- **Storage**: 500MB free space

### Recommended Development Environment
- **VS Code** with Nagari LSP extension
- **Git** for version control
- **Docker** (optional, for containerized development)

## Installation Methods

### Method 1: NPM Package (Recommended)

The easiest way to get started with Nagari:

```bash
# Global installation
npm install -g nagari-runtime

# Verify installation
nagari --version
```

### Method 2: Cargo Installation

For the latest development version:

```bash
# Install from cargo
cargo install --git https://github.com/ayanalamMOON/Nagari --bin nagari

# Or clone and build locally
git clone https://github.com/ayanalamMOON/Nagari.git
cd Nagari
cargo build --release
```

### Method 3: Pre-built Binaries

Download pre-built binaries from [GitHub Releases](https://github.com/ayanalamMOON/Nagari/releases):

1. Download the appropriate binary for your platform
2. Extract to a directory in your PATH
3. Make executable (Linux/macOS): `chmod +x nagari`

## Platform-Specific Instructions

### Windows

```powershell
# Using Windows Package Manager
winget install Nagari.Nagari

# Or using Chocolatey
choco install nagari

# Manual installation
# 1. Download from releases
# 2. Extract to C:\Program Files\Nagari
# 3. Add to PATH environment variable
```

### macOS

```bash
# Using Homebrew
brew install nagari

# Using MacPorts
sudo port install nagari

# Manual installation
curl -L https://github.com/ayanalamMOON/Nagari/releases/latest/download/nagari-macos.tar.gz | tar xz
sudo mv nagari /usr/local/bin/
```

### Linux

```bash
# Ubuntu/Debian
wget https://github.com/ayanalamMOON/Nagari/releases/latest/download/nagari-linux.deb
sudo dpkg -i nagari-linux.deb

# Arch Linux
yay -S nagari

# Manual installation
curl -L https://github.com/ayanalamMOON/Nagari/releases/latest/download/nagari-linux.tar.gz | tar xz
sudo mv nagari /usr/local/bin/
```

## Development Environment Setup

### VS Code Integration

1. Install the Nagari LSP extension:
   ```bash
   code --install-extension nagari.nagari-lsp
   ```

2. Configure VS Code settings:
   ```json
   {
     "nagari.lsp.enabled": true,
     "nagari.lsp.serverPath": "nagari-lsp",
     "files.associations": {
       "*.nag": "nagari"
     }
   }
   ```

### REPL Setup

The Nagari REPL provides interactive development:

```bash
# Start the REPL
nagari repl

# REPL with specific runtime
nagari repl --runtime node

# REPL with debugging
nagari repl --debug
```

## Verification

Verify your installation with these commands:

```bash
# Check version
nagari --version

# Run a simple program
echo 'console.log("Hello, Nagari!");' > hello.nag
nagari run hello.nag

# Start the REPL
nagari repl

# Run tests (if built from source)
cargo test
```

## Package Manager

### Setting up NagPkg

```bash
# Initialize a new project
nagari init my-project
cd my-project

# Install dependencies
nagari install

# Add a package
nagari add math@latest
```

## Troubleshooting Installation

### Common Issues

**Error: Command not found**
- Ensure Nagari is in your PATH
- On Windows, restart your terminal after installation

**Error: Permission denied**
- Use `sudo` on Linux/macOS for global installation
- On Windows, run as Administrator

**Error: Version mismatch**
- Update Node.js to the latest LTS version
- Clear npm cache: `npm cache clean --force`

**Error: Build failed**
- Ensure Rust toolchain is up to date: `rustup update`
- Install build dependencies: `sudo apt-get install build-essential` (Linux)

### Getting Help

- 📖 [Documentation](index.md)
- 🐛 [Issue Tracker](https://github.com/ayanalamMOON/Nagari/issues)
- 💬 [Discussions](https://github.com/ayanalamMOON/Nagari/discussions)
- 📧 [Support Email](mailto:support@nagari.dev)

## Next Steps

After installation:

1. **[Getting Started Guide](getting-started.md)** - Your first Nagari program
2. **[Language Tutorial](tutorials.md)** - Learn Nagari syntax
3. **[API Reference](api-reference.md)** - Explore the standard library
4. **[Examples](../examples/)** - Browse example code

---

*Installation taking too long? Check our [FAQ](faq.md) for common solutions.*

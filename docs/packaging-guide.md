# Nagari Executable Package Distribution

This document describes how to create and distribute standalone executable packages of the Nagari Programming Language for external use.

## Overview

The Nagari packaging system creates self-contained, executable packages that users can install without needing to compile from source. These packages include:

- Pre-compiled binaries for the target platform
- Complete runtime environment
- Standard library
- Documentation and examples
- Installation scripts

## Packaging Scripts

### 1. Single Platform Package (`package-release.sh` / `package-release.bat`)

Creates a package for a specific target platform.

**Usage:**
```bash
# Unix/Linux/macOS
./scripts/package-release.sh [version] [target]

# Windows
scripts\package-release.bat [version] [target]
```

**Examples:**
```bash
# Current platform, default version
./scripts/package-release.sh

# Specific version and target
./scripts/package-release.sh 0.3.0 x86_64-unknown-linux-gnu

# Windows build
./scripts/package-release.sh 0.3.0 x86_64-pc-windows-msvc
```

**Output:**
- `packages/nagari-[version]-[target]/` - Package directory
- `packages/nagari-[version]-[target].tar.gz` - Archive (Unix)
- `packages/nagari-[version]-[target].zip` - Archive (Windows)
- `packages/nagari-[version]-[target].[ext].sha256` - Checksum

### 2. Cross-Platform Package (`package-cross-platform.sh`)

Creates packages for all supported platforms in one run.

**Usage:**
```bash
./scripts/package-cross-platform.sh [version]
```

**Supported Targets:**
- `x86_64-unknown-linux-gnu` - Linux x64
- `x86_64-pc-windows-msvc` - Windows x64
- `x86_64-apple-darwin` - macOS Intel
- `aarch64-apple-darwin` - macOS Apple Silicon
- `aarch64-unknown-linux-gnu` - Linux ARM64

**Output:**
- `packages/cross-platform/` - All packages
- `packages/cross-platform/DISTRIBUTION.md` - Summary report

## Package Contents

Each package contains:

```
nagari-[version]-[target]/
├── bin/
│   ├── nag[.exe]           # Main CLI tool
│   ├── nagari-lsp[.exe]    # Language Server
│   └── nagc[.exe]          # Direct compiler (if available)
├── runtime/
│   ├── dist/               # Pre-compiled TypeScript runtime
│   └── package.json        # Runtime package info
├── stdlib/
│   └── *.nag              # Standard library modules
├── examples/
│   └── *.nag              # Example programs
├── docs/
│   ├── getting-started.md  # Quick start guide
│   ├── language-guide.md   # Syntax reference
│   └── cli-reference.md    # CLI documentation
├── install.sh/.bat         # Installation script
├── uninstall.sh/.bat       # Uninstallation script
├── README.md               # Package-specific instructions
├── PACKAGE_INFO.md         # Detailed package information
├── LICENSE                 # License file
└── CHANGELOG.md            # Version history (if available)
```

## Installation Process

### For End Users

1. **Download** the appropriate package for your platform
2. **Extract** the archive to a temporary location
3. **Run** the installation script:
   - Unix/Linux/macOS: `./install.sh`
   - Windows: `install.bat`
4. **Verify** installation: `nag --version`

### Installation Locations

- **Unix/Linux/macOS**: `~/.nagari/`
- **Windows**: `%USERPROFILE%\.nagari\`

### PATH Configuration

The installer will prompt users to add the binary directory to their PATH:
- **Unix**: `export PATH="$HOME/.nagari/bin:$PATH"`
- **Windows**: Add `%USERPROFILE%\.nagari\bin` to system PATH

## Development Workflow

### Prerequisites

Before creating packages, ensure:

1. **Rust toolchain** is installed and up-to-date
2. **Node.js/npm** is available for runtime building
3. **All tests pass**: `cargo test --workspace`
4. **Runtime builds**: `cd nagari-runtime && npm run build`

### Building Packages

#### For Development Testing
```bash
# Quick single-platform build
./scripts/package-release.sh

# Test the package
cd packages/nagari-0.3.0-x86_64-unknown-linux-gnu
./install.sh
nag --version
```

#### For Release Distribution
```bash
# Build all platforms
./scripts/package-cross-platform.sh 0.3.0

# Review distribution summary
cat packages/cross-platform/DISTRIBUTION.md

# Upload archives to GitHub Releases
```

### Quality Assurance

Each packaging script includes:

1. **Compilation verification** - Ensures all binaries build successfully
2. **Functionality testing** - Tests basic CLI operations
3. **Package integrity** - Verifies all required files are included
4. **Checksum generation** - Creates SHA256 hashes for verification

## Distribution Strategies

### 1. GitHub Releases

Upload packages as release assets:

```bash
# After running package-cross-platform.sh
cd packages/cross-platform

# Upload all .tar.gz, .zip, and .sha256 files
# to GitHub Releases page
```

### 2. Direct Download

Host packages on a web server or CDN:

```bash
# Example download URLs
https://releases.nagari-lang.org/v0.3.0/nagari-0.3.0-x86_64-unknown-linux-gnu.tar.gz
https://releases.nagari-lang.org/v0.3.0/nagari-0.3.0-x86_64-pc-windows-msvc.zip
```

### 3. Package Managers (Future)

Consider integration with:
- **Homebrew** (macOS/Linux)
- **Chocolatey** (Windows)
- **Snap** (Linux)
- **AppImage** (Linux)

## User Documentation

### Installation Instructions

Create user-friendly installation guides:

#### Linux/macOS
```bash
# Download and install
curl -L https://github.com/ayanalamMOON/Nagari/releases/download/v0.3.0/nagari-0.3.0-x86_64-unknown-linux-gnu.tar.gz | tar xz
cd nagari-0.3.0-x86_64-unknown-linux-gnu
./install.sh

# Verify
nag --version
```

#### Windows
```powershell
# Download from GitHub Releases
# Extract nagari-0.3.0-x86_64-pc-windows-msvc.zip
# Run install.bat
# Verify: nag --version
```

### Getting Started

After installation, users can:

```bash
# Run an example
nag run ~/.nagari/examples/hello.nag

# Create a new file
echo 'print("Hello, Nagari!")' > hello.nag
nag run hello.nag

# Compile to JavaScript
nag build hello.nag
```

## Troubleshooting

### Common Issues

1. **Missing PATH** - Binary not found after installation
   - Solution: Ensure `~/.nagari/bin` is in PATH

2. **Node.js dependency** - Runtime features not working
   - Solution: Install Node.js v16+ for full functionality

3. **Permission errors** - Installation script fails
   - Solution: Run with appropriate permissions

4. **Architecture mismatch** - Binary won't run
   - Solution: Download correct platform-specific package

### Build Issues

1. **Target not available** - Rust target missing
   - Solution: `rustup target add [target]`

2. **Cross-compilation fails** - Missing toolchain
   - Solution: Install platform-specific toolchain

3. **Runtime build fails** - npm/Node.js issues
   - Solution: Update Node.js, clear npm cache

## Security Considerations

### Package Integrity

- **SHA256 checksums** provided for all packages
- **Signed releases** (recommended for production)
- **Reproducible builds** where possible

### User Safety

- **No automatic PATH modification** - User must explicitly add
- **Local installation** - No system-wide changes required
- **Easy uninstall** - Uninstall script provided

## Maintenance

### Regular Tasks

1. **Update dependencies** in build scripts
2. **Test on all platforms** before release
3. **Maintain documentation** consistency
4. **Monitor user feedback** and issues

### Version Management

- **Semantic versioning** for releases
- **Changelog maintenance** for user awareness
- **Backward compatibility** considerations

## Future Improvements

### Planned Features

1. **Automatic updates** - Self-updating binaries
2. **Package managers** - Native OS integration
3. **Docker images** - Containerized distribution
4. **WebAssembly** - Browser-based execution
5. **IDE plugins** - Editor integration packages

### Performance Optimizations

1. **Smaller binaries** - Strip debug symbols
2. **Faster compression** - Better archive formats
3. **Incremental updates** - Delta packages
4. **CDN distribution** - Global availability

---

This packaging system ensures that Nagari can be easily distributed and installed by users without requiring development tools or compilation knowledge.

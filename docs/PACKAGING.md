# Nagari Executable Package Creation

This directory contains scripts and tools for creating standalone executable packages of the Nagari Programming Language for external distribution.

## Quick Start

### Simple Package Creation

```bash
# Build package for current platform
./package.sh single

# Build package for specific platform
./package.sh single 0.3.0 linux
./package.sh single 0.3.0 windows
./package.sh single 0.3.0 macos

# Build packages for all platforms
./package.sh multi 0.3.0

# Test build
./package.sh test
```

### Manual Package Creation

```bash
# Single platform (advanced)
./scripts/package-release.sh [version] [target]

# Cross-platform (advanced)
./scripts/package-cross-platform.sh [version]
```

## What Gets Created

Each package includes:
- ✅ **Pre-compiled binaries** (nag, nagari-lsp)
- ✅ **Runtime environment** (JavaScript/TypeScript)
- ✅ **Standard library** (all .nag modules)
- ✅ **Documentation** (getting started, language guide)
- ✅ **Examples** (sample programs)
- ✅ **Installation scripts** (install.sh/install.bat)
- ✅ **Uninstall scripts** (uninstall.sh/uninstall.bat)

## Supported Platforms

| Platform            | Target                      | Archive Format |
| ------------------- | --------------------------- | -------------- |
| Linux x64           | `x86_64-unknown-linux-gnu`  | `.tar.gz`      |
| Windows x64         | `x86_64-pc-windows-msvc`    | `.zip`         |
| macOS Intel         | `x86_64-apple-darwin`       | `.tar.gz`      |
| macOS Apple Silicon | `aarch64-apple-darwin`      | `.tar.gz`      |
| Linux ARM64         | `aarch64-unknown-linux-gnu` | `.tar.gz`      |

## Package Structure

```
nagari-0.3.0-x86_64-unknown-linux-gnu/
├── bin/
│   ├── nag                    # Main CLI tool
│   └── nagari-lsp             # Language Server
├── runtime/                   # JavaScript runtime
├── stdlib/                    # Standard library
├── examples/                  # Example programs
├── docs/                      # Documentation
├── install.sh                 # Installation script
├── uninstall.sh               # Uninstallation script
├── README.md                  # Package instructions
├── PACKAGE_INFO.md            # Detailed information
└── LICENSE                    # License file
```

## Installation for End Users

### Unix/Linux/macOS
```bash
# Download and extract package
tar -xzf nagari-0.3.0-x86_64-unknown-linux-gnu.tar.gz
cd nagari-0.3.0-x86_64-unknown-linux-gnu

# Install
./install.sh

# Verify
nag --version
```

### Windows
```cmd
# Extract nagari-0.3.0-x86_64-pc-windows-msvc.zip
# Navigate to extracted folder
install.bat

# Verify
nag --version
```

## Development Workflow

### Prerequisites
1. **Rust toolchain** - Latest stable
2. **Node.js/npm** - v16+ for runtime building
3. **Platform targets** - Installed via rustup

### Building Process
1. **Test build** - `./package.sh test`
2. **Single platform** - `./package.sh single`
3. **All platforms** - `./package.sh multi`
4. **Quality check** - Test packages manually
5. **Distribution** - Upload to GitHub Releases

### Quality Assurance
- ✅ All binaries compile successfully
- ✅ Basic functionality tests pass
- ✅ Package integrity verification
- ✅ SHA256 checksums generated
- ✅ Installation scripts tested

## File Locations

| Script                              | Purpose                         |
| ----------------------------------- | ------------------------------- |
| `package.sh`                        | Simple wrapper interface        |
| `scripts/package-release.sh`        | Single platform builder         |
| `scripts/package-release.bat`       | Windows single platform builder |
| `scripts/package-cross-platform.sh` | Multi-platform builder          |
| `docs/packaging-guide.md`           | Complete documentation          |

## Output Locations

| Build Type      | Output Directory                      |
| --------------- | ------------------------------------- |
| Single platform | `packages/nagari-[version]-[target]/` |
| Cross-platform  | `packages/cross-platform/`            |
| Archives        | Same as package directories           |
| Checksums       | `[archive].sha256`                    |

## Troubleshooting

### Common Issues

**Build fails for target**
```bash
# Install missing target
rustup target add x86_64-unknown-linux-gnu
```

**Runtime build fails**
```bash
# Update Node.js and rebuild
cd nagari-runtime
npm install
npm run build
```

**Permission denied**
```bash
# Make scripts executable
chmod +x package.sh scripts/*.sh
```

**Missing dependencies**
```bash
# Install required tools
# Rust: https://rustup.rs/
# Node.js: https://nodejs.org/
```

### Platform-Specific Issues

**macOS Code Signing**
- Packages are not signed by default
- Users may need to allow in Security preferences

**Windows Defender**
- May flag unsigned executables
- Consider code signing for production

**Linux Dependencies**
- Packages are statically linked
- Should work on most distributions

## Distribution Best Practices

### For GitHub Releases
1. Create release tag: `v0.3.0`
2. Upload all platform archives
3. Include checksums for verification
4. Provide clear installation instructions

### For Direct Distribution
1. Host on reliable CDN
2. Provide download verification
3. Maintain consistent URLs
4. Monitor download analytics

### Documentation
1. Clear installation instructions
2. Platform-specific guides
3. Troubleshooting sections
4. Getting started examples

## Security Considerations

- 🔒 **SHA256 checksums** for integrity verification
- 🔒 **No automatic PATH modification** - user controlled
- 🔒 **Local installation** - no system-wide changes
- 🔒 **Easy uninstall** - complete removal possible

## Next Steps

After creating packages:

1. **Test installation** on target platforms
2. **Upload to GitHub Releases** or distribution platform
3. **Update documentation** with download links
4. **Announce release** to community
5. **Monitor feedback** and issues

---

For complete documentation, see [`docs/packaging-guide.md`](docs/packaging-guide.md).

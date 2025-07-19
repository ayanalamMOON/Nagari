# Nagari Release Process

This document describes how to build and release binary packages for the Nagari Programming Language.

## Quick Start

### Automated Release (Recommended)

1. **Prepare for release:**
   ```bash
   # Run tests to ensure everything works
   cargo test --workspace

   # Build and test runtime
   cd nagari-runtime && npm test && cd ..
   ```

2. **Create a release:**
   ```bash
   # Using the release script (Unix/macOS)
   ./scripts/release.sh 0.3.0

   # Or on Windows
   scripts\release.bat 0.3.0
   ```

3. **Monitor the release:**
   - Check GitHub Actions workflow progress
   - Verify the release appears on GitHub Releases page
   - Test download and installation of binary packages

### Manual Release

If you need to create a release manually:

1. **Build locally:**
   ```bash
   # Build for current platform
   ./scripts/build.sh

   # Build for specific target
   ./scripts/build.sh x86_64-unknown-linux-gnu
   ```

2. **Create and push tag:**
   ```bash
   git tag -a v0.3.0 -m "Release v0.3.0"
   git push origin v0.3.0
   ```

## Release Workflow

### Automated GitHub Actions

The release process is automated through GitHub Actions workflows:

#### 1. **Release Workflow** (`.github/workflows/release.yml`)
- **Trigger:** Push tag matching `v*` pattern or manual dispatch
- **Builds:** Cross-platform binaries for Windows, macOS, and Linux
- **Outputs:**
  - GitHub Release with binary packages
  - Automated npm publishing of nagari-runtime

#### 2. **CI Workflow** (`.github/workflows/ci.yml`)
- **Trigger:** Push to main/develop branches, pull requests
- **Tests:** Cross-platform testing, security audit, benchmarks
- **Outputs:** Test results and documentation

#### 3. **Build Test Workflow** (`.github/workflows/build-test.yml`)
- **Trigger:** Pull requests affecting build files
- **Tests:** Build verification and binary size analysis
- **Outputs:** Build artifacts and size reports

### Supported Platforms

The release workflow builds binaries for:

| Platform | Target | Archive Format |
|----------|--------|----------------|
| Windows x64 | `x86_64-pc-windows-msvc` | `.zip` |
| macOS x64 | `x86_64-apple-darwin` | `.tar.gz` |
| macOS ARM64 | `aarch64-apple-darwin` | `.tar.gz` |
| Linux x64 | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| Linux x64 (musl) | `x86_64-unknown-linux-musl` | `.tar.gz` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `.tar.gz` |

### Release Package Contents

Each release package includes:

```
nagari-v0.3.0-<target>/
├── nag(.exe)                    # CLI tool
├── nagari-lsp(.exe)             # LSP server
├── stdlib/                      # Standard library
│   ├── core.nag
│   ├── crypto.nag
│   └── ...
├── runtime/                     # Runtime package
│   ├── index.js
│   ├── package.json
│   └── ...
├── README.md                    # Documentation
├── LICENSE                      # License file
├── CHANGELOG.md                 # Change history
└── install(.sh|.bat)           # Installation script
```

## Local Development Builds

### Build Script Usage

```bash
# Build for current platform
./scripts/build.sh

# Build for specific target
./scripts/build.sh x86_64-pc-windows-msvc
./scripts/build.sh x86_64-apple-darwin
./scripts/build.sh x86_64-unknown-linux-gnu

# On Windows
scripts\build.bat
scripts\build.bat x86_64-pc-windows-msvc
```

### Manual Build Process

1. **Install dependencies:**
   ```bash
   # Install Rust target
   rustup target add x86_64-unknown-linux-gnu

   # Install Node.js dependencies
   cd nagari-runtime && npm install && cd ..
   ```

2. **Build runtime:**
   ```bash
   cd nagari-runtime
   npm run build
   cd ..
   ```

3. **Build Rust binaries:**
   ```bash
   cargo build --release --target x86_64-unknown-linux-gnu
   ```

4. **Test build:**
   ```bash
   ./target/x86_64-unknown-linux-gnu/release/nag --version
   ```

## Release Checklist

Before creating a release:

- [ ] All tests pass (`cargo test --workspace`)
- [ ] Runtime tests pass (`cd nagari-runtime && npm test`)
- [ ] Version updated in `nagari-runtime/package.json`
- [ ] CHANGELOG.md updated with release notes
- [ ] Documentation is up to date
- [ ] No uncommitted changes in git

### Version Numbering

Nagari follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version: Incompatible API changes
- **MINOR** version: New functionality (backwards compatible)
- **PATCH** version: Bug fixes (backwards compatible)

Examples:
- `v0.3.0` - Minor release with new features
- `v0.3.1` - Patch release with bug fixes
- `v1.0.0` - Major release (stable API)

## GitHub Secrets Setup

For automated npm publishing, configure these GitHub secrets:

| Secret | Description |
|--------|-------------|
| `NPM_TOKEN` | npm authentication token for publishing |
| `GITHUB_TOKEN` | Automatically provided by GitHub Actions |

## Troubleshooting

### Common Issues

1. **Build fails on specific target:**
   - Ensure cross-compilation tools are installed
   - Check if target is properly added with `rustup target add`

2. **npm publish fails:**
   - Verify `NPM_TOKEN` secret is set
   - Check if version already exists on npm
   - Ensure package.json version is updated

3. **Git tag already exists:**
   - Delete local tag: `git tag -d v0.3.0`
   - Delete remote tag: `git push origin --delete v0.3.0`
   - Recreate tag with correct version

### Debug Builds

For debugging build issues:

```bash
# Enable verbose output
RUST_LOG=debug cargo build --release --verbose

# Check binary dependencies
ldd target/release/nag  # Linux
otool -L target/release/nag  # macOS
```

## Contributing to Releases

1. **Test your changes** on multiple platforms before creating PR
2. **Update CHANGELOG.md** with your changes
3. **Increment version** appropriately in package.json
4. **Add tests** for new functionality
5. **Update documentation** as needed

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Cross-compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [npm Publishing Guide](https://docs.npmjs.com/creating-and-publishing-unscoped-public-packages)

# Nagari Project Structure

This document describes the organization of the Nagari programming language repository.

## Root Directory Structure

```
Nagari/
├── .github/                 # GitHub-specific files and community documents
│   ├── workflows/          # GitHub Actions CI/CD workflows
│   ├── CODE_OF_CONDUCT.md  # Community code of conduct
│   ├── CONTRIBUTING.md     # Contribution guidelines
│   ├── FUNDING.yml         # Sponsor configuration
│   ├── SEO_CONFIG.md       # SEO strategy and configuration
│   └── TOPICS.md           # Repository topic strategy
├── docs/                   # Documentation and guides
├── examples/               # Comprehensive language examples
├── samples/               # Simple test files and basic samples
├── scripts/               # Build and utility scripts
├── specs/                 # Language specifications
├── src/                   # Source code
├── stdlib/                # Standard library
├── tests/                 # Test suites
├── test-files/            # Test data and fixtures
├── vscode-extension/      # VS Code extension
├── web/                   # Web assets and SEO files
├── dev-tools/             # Development tools
├── nagari-runtime/        # Runtime library
├── nagari-runtime-global/ # Global runtime distribution
├── README.md              # Main project documentation
├── CHANGELOG.md           # Version history
├── CREDITS.md             # Contributors and acknowledgments
├── TODO.md                # Development roadmap
├── LICENSE                # Project license
├── Cargo.toml             # Rust project configuration
└── .gitignore             # Git ignore rules
```

## Directory Descriptions

### Core Development
- **`src/`** - Main Rust source code for compiler, parser, VM, and tools
- **`stdlib/`** - Standard library modules (.nag files)
- **`specs/`** - Language specifications and grammar definitions

### Documentation & Examples
- **`docs/`** - Comprehensive documentation and guides
- **`examples/`** - Full-featured example programs demonstrating language capabilities
- **`samples/`** - Simple test files and basic language samples
- **`web/`** - SEO-optimized web assets and landing page

### Testing & Quality
- **`tests/`** - Automated test suites
- **`test-files/`** - Test data, fixtures, and integration tests

### Tooling & Development
- **`scripts/`** - Build scripts, automation, and utilities
- **`dev-tools/`** - Development tools and helpers
- **`vscode-extension/`** - VS Code language support extension

### Distribution & Runtime
- **`nagari-runtime/`** - TypeScript runtime library
- **`nagari-runtime-global/`** - Global runtime distribution

### Build Artifacts (Ignored)
- **`target/`** - Rust build outputs (ignored by git)
- **`dist/`** - Transpiled JavaScript outputs (ignored by git)

## File Organization Principles

1. **Separation of Concerns**: Each directory has a specific purpose
2. **User Experience**: Easy navigation for different user types (contributors, users, learners)
3. **Build Cleanliness**: Build artifacts are properly ignored and separated
4. **SEO Optimization**: Web assets organized for deployment and discoverability
5. **Community Standards**: GitHub community files in standard locations

## Quick Navigation

- **Getting Started**: Start with `README.md` and `docs/getting-started.md`
- **Contributing**: See `.github/CONTRIBUTING.md`
- **Examples**: Browse `examples/` for comprehensive demos, `samples/` for simple tests
- **Documentation**: Full docs in `docs/` directory
- **Language Spec**: See `specs/language-spec.md`
- **Building**: Use scripts in `scripts/` directory

## Maintenance

- Build artifacts in `target/` and `dist/` are automatically cleaned by CI
- Sample files in `samples/` are used for development testing
- Web assets in `web/` are deployment-ready for GitHub Pages
- Documentation is automatically checked for consistency

This structure ensures the project is professional, navigable, and follows modern open-source best practices.
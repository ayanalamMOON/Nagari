#!/bin/bash

# Nagari Release Preparation Tool
# Comprehensive release preparation and packaging

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_DIR="$PROJECT_ROOT/release"
DIST_DIR="$PROJECT_ROOT/dist"
DOCS_DIR="$PROJECT_ROOT/docs"

# Release options
BUILD_RELEASE=true
RUN_TESTS=true
GENERATE_DOCS=true
CREATE_PACKAGES=true
DRY_RUN=false
SKIP_CHECKS=false

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  Nagari Release Preparation${NC}"
    echo -e "${BLUE}================================${NC}"
    echo
}

print_section() {
    echo
    echo -e "${PURPLE}--- $1 ---${NC}"
}

print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

get_version() {
    if [ -f "$PROJECT_ROOT/Cargo.toml" ]; then
        grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | sed 's/version = "\(.*\)"/\1/'
    else
        echo "unknown"
    fi
}

setup_release_environment() {
    print_section "Setup Release Environment"
    print_info "Preparing release environment..."

    # Clean and create release directories
    rm -rf "$RELEASE_DIR"
    mkdir -p "$RELEASE_DIR"/{binaries,packages,docs,checksums}

    # Clean dist directory
    rm -rf "$DIST_DIR"
    mkdir -p "$DIST_DIR"

    cd "$PROJECT_ROOT"

    print_success "Release environment ready"
}

run_pre_release_checks() {
    if $SKIP_CHECKS; then
        print_info "Skipping pre-release checks"
        return 0
    fi

    print_section "Pre-Release Checks"

    # Check git status
    if [ -d ".git" ]; then
        print_info "Checking git status..."
        if ! git diff-index --quiet HEAD --; then
            print_warning "Working directory has uncommitted changes"
            if ! $DRY_RUN; then
                read -p "Continue anyway? (y/N) " -n 1 -r
                echo
                if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                    print_info "Aborting release preparation"
                    exit 0
                fi
            fi
        else
            print_success "Working directory is clean"
        fi
    fi

    # Check for required files
    local required_files=("README.md" "LICENSE" "CHANGELOG.md" "Cargo.toml")
    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            print_error "Required file missing: $file"
            exit 1
        fi
    done
    print_success "Required files present"

    # Check version in changelog
    local version=$(get_version)
    if ! grep -q "## \[$version\]" CHANGELOG.md; then
        print_warning "Version $version not found in CHANGELOG.md"
        print_info "Consider updating the changelog before release"
    else
        print_success "Version found in changelog"
    fi

    print_success "Pre-release checks completed"
}

run_tests() {
    if ! $RUN_TESTS; then
        print_info "Skipping tests"
        return 0
    fi

    print_section "Running Tests"
    print_info "Running comprehensive test suite..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would run tests"
        return 0
    fi

    # Run the test suite
    if [ -x "$PROJECT_ROOT/dev-tools/test-runner.sh" ]; then
        "$PROJECT_ROOT/dev-tools/test-runner.sh" --coverage
    else
        cargo test --release
    fi

    print_success "All tests passed"
}

build_release_binaries() {
    if ! $BUILD_RELEASE; then
        print_info "Skipping release build"
        return 0
    fi

    print_section "Building Release Binaries"
    print_info "Building optimized release binaries..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would build release binaries"
        return 0
    fi

    # Build release version
    cargo build --release --verbose

    # Copy binaries to release directory
    if [ -f "target/release/nag" ]; then
        cp "target/release/nag" "$RELEASE_DIR/binaries/"
        print_success "Copied nag binary"
    fi

    if [ -f "target/release/nag.exe" ]; then
        cp "target/release/nag.exe" "$RELEASE_DIR/binaries/"
        print_success "Copied nag.exe binary"
    fi

    # Build additional components
    for component in lsp-server nagari-vm nagari-embedded; do
        if [ -d "src/$component" ]; then
            print_info "Building $component..."
            cargo build --release --package "$component" || print_warning "$component build failed"
        fi
    done

    print_success "Release binaries built"
}

build_runtime_packages() {
    print_section "Building Runtime Packages"

    # Build TypeScript runtime
    if [ -d "nagari-runtime" ]; then
        print_info "Building TypeScript runtime..."

        if $DRY_RUN; then
            print_info "[DRY RUN] Would build TypeScript runtime"
        else
            cd nagari-runtime
            if [ -f "package.json" ]; then
                npm install
                npm run build
                npm pack
                cp nagari-runtime-*.tgz "$RELEASE_DIR/packages/"
                print_success "Built TypeScript runtime package"
            fi
            cd "$PROJECT_ROOT"
        fi
    fi

    # Build global runtime
    if [ -d "nagari-runtime-global" ]; then
        print_info "Building global runtime..."

        if $DRY_RUN; then
            print_info "[DRY RUN] Would build global runtime"
        else
            cd nagari-runtime-global
            if [ -f "package.json" ]; then
                npm install
                npm run build
                npm pack
                cp nagari-runtime-global-*.tgz "$RELEASE_DIR/packages/"
                print_success "Built global runtime package"
            fi
            cd "$PROJECT_ROOT"
        fi
    fi

    print_success "Runtime packages built"
}

build_vscode_extension() {
    print_section "Building VS Code Extension"

    if [ ! -d "vscode-extension" ]; then
        print_info "VS Code extension not found, skipping..."
        return 0
    fi

    print_info "Building VS Code extension..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would build VS Code extension"
        return 0
    fi

    cd vscode-extension

    if [ -f "package.json" ]; then
        npm install

        # Install vsce if not present
        if ! command -v vsce &> /dev/null; then
            npm install -g vsce
        fi

        # Package extension
        vsce package
        cp *.vsix "$RELEASE_DIR/packages/"
        print_success "Built VS Code extension"
    else
        print_warning "package.json not found in vscode-extension"
    fi

    cd "$PROJECT_ROOT"
}

generate_documentation() {
    if ! $GENERATE_DOCS; then
        print_info "Skipping documentation generation"
        return 0
    fi

    print_section "Generating Documentation"
    print_info "Generating release documentation..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would generate documentation"
        return 0
    fi

    # Generate Rust documentation
    cargo doc --release --no-deps

    # Copy documentation
    if [ -d "target/doc" ]; then
        cp -r target/doc "$RELEASE_DIR/docs/rust-api"
        print_success "Copied Rust API documentation"
    fi

    # Copy user documentation
    if [ -d "$DOCS_DIR" ]; then
        cp -r "$DOCS_DIR" "$RELEASE_DIR/docs/user-guide"
        print_success "Copied user documentation"
    fi

    # Generate README for release
    cat > "$RELEASE_DIR/README.md" << EOF
# Nagari Programming Language Release

Version: $(get_version)
Release Date: $(date '+%Y-%m-%d')

## Contents

### Binaries
- \`binaries/\` - Compiled executables for various platforms

### Packages
- \`packages/\` - Distributable packages (npm, crates.io, VS Code extension)

### Documentation
- \`docs/rust-api/\` - Rust API documentation
- \`docs/user-guide/\` - User documentation and guides

### Checksums
- \`checksums/\` - SHA256 checksums for all release files

## Installation

See the main project README.md for installation instructions.

## Verification

Verify file integrity using the checksums in the \`checksums/\` directory:

\`\`\`bash
sha256sum -c checksums/SHA256SUMS
\`\`\`

## Support

- GitHub: https://github.com/ayanalamMOON/Nagari
- Issues: https://github.com/ayanalamMOON/Nagari/issues
- Documentation: https://github.com/ayanalamMOON/Nagari/tree/main/docs
EOF

    print_success "Documentation generated"
}

create_distribution_packages() {
    if ! $CREATE_PACKAGES; then
        print_info "Skipping package creation"
        return 0
    fi

    print_section "Creating Distribution Packages"

    local version=$(get_version)

    # Create source package
    print_info "Creating source package..."
    if $DRY_RUN; then
        print_info "[DRY RUN] Would create source package"
    else
        git archive --format=tar.gz --prefix="nagari-$version/" HEAD > "$RELEASE_DIR/packages/nagari-$version-src.tar.gz"
        print_success "Created source package"
    fi

    # Create binary packages for different platforms
    print_info "Creating binary packages..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would create binary packages"
    else
        # Linux/Unix package
        if [ -f "$RELEASE_DIR/binaries/nag" ]; then
            cd "$RELEASE_DIR"
            tar -czf "packages/nagari-$version-linux-x86_64.tar.gz" -C binaries nag
            print_success "Created Linux binary package"
        fi

        # Windows package
        if [ -f "$RELEASE_DIR/binaries/nag.exe" ]; then
            cd "$RELEASE_DIR"
            zip -q "packages/nagari-$version-windows-x86_64.zip" binaries/nag.exe
            print_success "Created Windows binary package"
        fi

        cd "$PROJECT_ROOT"
    fi
}

generate_checksums() {
    print_section "Generating Checksums"
    print_info "Generating SHA256 checksums..."

    if $DRY_RUN; then
        print_info "[DRY RUN] Would generate checksums"
        return 0
    fi

    cd "$RELEASE_DIR"

    # Generate checksums for all files
    find . -type f -not -path "./checksums/*" -exec sha256sum {} \; > checksums/SHA256SUMS

    # Sort the checksums file
    sort checksums/SHA256SUMS -o checksums/SHA256SUMS

    print_success "Checksums generated"

    cd "$PROJECT_ROOT"
}

create_release_notes() {
    print_section "Creating Release Notes"
    print_info "Generating release notes..."

    local version=$(get_version)
    local release_notes="$RELEASE_DIR/RELEASE_NOTES.md"

    if $DRY_RUN; then
        print_info "[DRY RUN] Would create release notes"
        return 0
    fi

    cat > "$release_notes" << EOF
# Nagari $version Release Notes

$(date '+%B %d, %Y')

## What's New

EOF

    # Extract changelog for this version
    if [ -f "CHANGELOG.md" ]; then
        echo "## Changes" >> "$release_notes"
        echo "" >> "$release_notes"

        # Extract section for this version from changelog
        sed -n "/## \[$version\]/,/## \[/p" CHANGELOG.md | sed '$d' | tail -n +2 >> "$release_notes"
    fi

    cat >> "$release_notes" << EOF

## Download

### Binaries
- Linux: \`nagari-$version-linux-x86_64.tar.gz\`
- Windows: \`nagari-$version-windows-x86_64.zip\`

### Packages
- NPM Runtime: \`nagari-runtime-$version.tgz\`
- VS Code Extension: \`nagari-$version.vsix\`

### Source
- Source Code: \`nagari-$version-src.tar.gz\`

## Installation

### From Binary
\`\`\`bash
# Linux/macOS
tar -xzf nagari-$version-linux-x86_64.tar.gz
sudo mv nag /usr/local/bin/

# Windows
# Extract nagari-$version-windows-x86_64.zip
# Add nag.exe to your PATH
\`\`\`

### From Source
\`\`\`bash
tar -xzf nagari-$version-src.tar.gz
cd nagari-$version
cargo build --release
sudo cp target/release/nag /usr/local/bin/
\`\`\`

### NPM Runtime
\`\`\`bash
npm install -g nagari-runtime-$version.tgz
\`\`\`

## Verification

All release files include SHA256 checksums. Verify downloads:

\`\`\`bash
sha256sum -c SHA256SUMS
\`\`\`

## Support

- 📖 Documentation: [docs/](docs/)
- 🐛 Issues: [GitHub Issues](https://github.com/ayanalamMOON/Nagari/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/ayanalamMOON/Nagari/discussions)

---

Thank you for using Nagari! 🚀
EOF

    print_success "Release notes created"
}

show_release_summary() {
    local version=$(get_version)

    echo
    echo -e "${GREEN}==== RELEASE PREPARATION COMPLETE ====${NC}"
    echo -e "Version: ${YELLOW}$version${NC}"
    echo -e "Release directory: ${BLUE}$RELEASE_DIR${NC}"
    echo

    if $DRY_RUN; then
        echo -e "${YELLOW}DRY RUN MODE - No files were created${NC}"
    else
        echo -e "${YELLOW}Release Contents:${NC}"

        # Show directory structure
        if command -v tree &> /dev/null; then
            tree "$RELEASE_DIR"
        else
            find "$RELEASE_DIR" -type f | sort
        fi

        echo
        echo -e "${YELLOW}Next Steps:${NC}"
        echo "1. Review release contents in: $RELEASE_DIR"
        echo "2. Test binaries and packages"
        echo "3. Update release notes if needed"
        echo "4. Create GitHub release and upload files"
        echo "5. Publish packages to registries:"
        echo "   • cargo publish (for Rust crate)"
        echo "   • npm publish (for runtime packages)"
        echo "   • vsce publish (for VS Code extension)"
        echo "6. Announce the release"

        # Calculate total size
        local total_size=$(du -sh "$RELEASE_DIR" | cut -f1)
        echo
        echo -e "Total release size: ${GREEN}$total_size${NC}"
    fi

    echo
}

main() {
    print_header

    local version=$(get_version)
    print_info "Preparing release for version: $version"

    setup_release_environment
    run_pre_release_checks
    run_tests
    build_release_binaries
    build_runtime_packages
    build_vscode_extension
    generate_documentation
    create_distribution_packages
    generate_checksums
    create_release_notes
    show_release_summary
}

# Help text
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Nagari Release Preparation Tool"
    echo
    echo "Comprehensive release preparation and packaging."
    echo
    echo "Usage: $0 [options]"
    echo
    echo "Options:"
    echo "  --dry-run          Show what would be done without creating files"
    echo "  --skip-tests       Skip running tests"
    echo "  --skip-build       Skip building release binaries"
    echo "  --skip-docs        Skip generating documentation"
    echo "  --skip-packages    Skip creating distribution packages"
    echo "  --skip-checks      Skip pre-release checks"
    echo "  --help, -h         Show this help message"
    echo
    echo "Features:"
    echo "  • Pre-release validation and checks"
    echo "  • Comprehensive test suite execution"
    echo "  • Optimized release binary compilation"
    echo "  • Runtime package building (TypeScript, npm)"
    echo "  • VS Code extension packaging"
    echo "  • Documentation generation"
    echo "  • Distribution package creation"
    echo "  • SHA256 checksum generation"
    echo "  • Release notes creation"
    echo
    echo "Output:"
    echo "  All release files are created in: release/"
    echo "  • binaries/     - Compiled executables"
    echo "  • packages/     - Distribution packages"
    echo "  • docs/         - Generated documentation"
    echo "  • checksums/    - SHA256 checksums"
    echo "  • README.md     - Release information"
    echo "  • RELEASE_NOTES.md - Detailed release notes"
    echo
    echo "Example:"
    echo "  $0                 Full release preparation"
    echo "  $0 --dry-run       Preview without creating files"
    echo "  $0 --skip-tests    Skip test execution"
    exit 0
fi

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-tests)
            RUN_TESTS=false
            shift
            ;;
        --skip-build)
            BUILD_RELEASE=false
            shift
            ;;
        --skip-docs)
            GENERATE_DOCS=false
            shift
            ;;
        --skip-packages)
            CREATE_PACKAGES=false
            shift
            ;;
        --skip-checks)
            SKIP_CHECKS=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

main "$@"

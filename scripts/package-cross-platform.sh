#!/bin/bash
# Cross-platform package builder for Nagari Programming Language
# Creates executable packages for multiple targets
# Usage: ./scripts/package-cross-platform.sh [version]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_step() { echo -e "${BLUE}🔷 $1${NC}"; }
print_success() { echo -e "${GREEN}✅ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️ $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }
print_info() { echo -e "${CYAN}ℹ️ $1${NC}"; }
print_package() { echo -e "${PURPLE}📦 $1${NC}"; }

# Configuration
VERSION=${1:-"0.3.0"}
ROOT_DIR=$(pwd)

# Supported targets
TARGETS=(
    "x86_64-unknown-linux-gnu"      # Linux x64
    "x86_64-pc-windows-msvc"        # Windows x64
    "x86_64-apple-darwin"           # macOS x64 Intel
    "aarch64-apple-darwin"          # macOS ARM64 (Apple Silicon)
    "aarch64-unknown-linux-gnu"     # Linux ARM64
)

print_step "Creating Nagari v${VERSION} cross-platform packages"

# Validate environment
if [ ! -f "Cargo.toml" ] || [ ! -d "src" ]; then
    print_error "This script must be run from the project root directory"
    exit 1
fi

# Check required tools
command -v rustc >/dev/null 2>&1 || { print_error "Rust is required but not installed"; exit 1; }
command -v cargo >/dev/null 2>&1 || { print_error "Cargo is required but not installed"; exit 1; }
command -v npm >/dev/null 2>&1 || { print_error "Node.js/npm is required but not installed"; exit 1; }

# Build runtime once (target-independent)
print_step "Building Nagari runtime (TypeScript/JavaScript)"
cd nagari-runtime

if [ ! -d "node_modules" ]; then
    npm install
fi

npm run build
print_success "Runtime built successfully"
cd "${ROOT_DIR}"

# Clean packages directory
rm -rf packages/cross-platform
mkdir -p packages/cross-platform

# Track successful builds
declare -a SUCCESSFUL_BUILDS=()
declare -a FAILED_BUILDS=()

# Build for each target
for target in "${TARGETS[@]}"; do
    print_step "Building for target: ${target}"

    # Install target
    if rustup target add "${target}"; then
        print_success "Target ${target} added successfully"
    else
        print_warning "Failed to add target ${target}, skipping..."
        FAILED_BUILDS+=("${target}")
        continue
    fi

    # Build the workspace
    if cargo build --release --target "${target}" --workspace; then
        print_success "Build successful for ${target}"

        # Package this target
        print_step "Packaging ${target}"

        PACKAGE_NAME="nagari-${VERSION}-${target}"
        PACKAGE_DIR="packages/cross-platform/${PACKAGE_NAME}"

        mkdir -p "${PACKAGE_DIR}"/{bin,runtime,stdlib,examples,docs}

        # Determine binary extensions
        CLI_BINARY="target/${target}/release/nag"
        LSP_BINARY="target/${target}/release/nagari-lsp"
        COMPILER_BINARY="target/${target}/release/nagc"

        if [[ "$target" == *"windows"* ]]; then
            CLI_BINARY="${CLI_BINARY}.exe"
            LSP_BINARY="${LSP_BINARY}.exe"
            COMPILER_BINARY="${COMPILER_BINARY}.exe"
        fi

        # Copy binaries if they exist
        if [ -f "$CLI_BINARY" ]; then
            cp "$CLI_BINARY" "${PACKAGE_DIR}/bin/"
        else
            print_warning "CLI binary not found for ${target}"
        fi

        if [ -f "$LSP_BINARY" ]; then
            cp "$LSP_BINARY" "${PACKAGE_DIR}/bin/"
        else
            print_warning "LSP binary not found for ${target}"
        fi

        if [ -f "$COMPILER_BINARY" ]; then
            cp "$COMPILER_BINARY" "${PACKAGE_DIR}/bin/"
        fi

        # Copy runtime (same for all targets)
        cp -r nagari-runtime/dist/* "${PACKAGE_DIR}/runtime/"
        cp nagari-runtime/package.json "${PACKAGE_DIR}/runtime/"

        # Copy standard library
        if [ -d "stdlib" ]; then
            cp -r stdlib/* "${PACKAGE_DIR}/stdlib/"
        fi

        # Copy examples
        if [ -d "examples" ]; then
            cp -r examples/* "${PACKAGE_DIR}/examples/"
        fi

        # Copy documentation
        cp README.md "${PACKAGE_DIR}/"
        cp LICENSE "${PACKAGE_DIR}/"
        if [ -f "CHANGELOG.md" ]; then
            cp CHANGELOG.md "${PACKAGE_DIR}/"
        fi

        # Copy key documentation files
        for doc in docs/getting-started.md docs/language-guide.md docs/cli-reference.md; do
            if [ -f "$doc" ]; then
                cp "$doc" "${PACKAGE_DIR}/docs/"
            fi
        done

        # Create target-specific installation script
        if [[ "$target" == *"windows"* ]]; then
            # Windows batch installer
            cat > "${PACKAGE_DIR}/install.bat" << 'EOF'
@echo off
echo 🚀 Installing Nagari Programming Language...

set "INSTALL_DIR=%USERPROFILE%\.nagari"
set "BIN_DIR=%INSTALL_DIR%\bin"
set "STDLIB_DIR=%INSTALL_DIR%\stdlib"
set "RUNTIME_DIR=%INSTALL_DIR%\runtime"
set "EXAMPLES_DIR=%INSTALL_DIR%\examples"

if not exist "%BIN_DIR%" mkdir "%BIN_DIR%"
if not exist "%STDLIB_DIR%" mkdir "%STDLIB_DIR%"
if not exist "%RUNTIME_DIR%" mkdir "%RUNTIME_DIR%"
if not exist "%EXAMPLES_DIR%" mkdir "%EXAMPLES_DIR%"

echo 📁 Copying binaries...
copy bin\*.exe "%BIN_DIR%\"

echo 📚 Copying standard library...
if exist "stdlib" xcopy /E /I /Q stdlib "%STDLIB_DIR%"

echo ⚡ Copying runtime...
if exist "runtime" xcopy /E /I /Q runtime "%RUNTIME_DIR%"

echo 📖 Copying examples...
if exist "examples" xcopy /E /I /Q examples "%EXAMPLES_DIR%"

echo 📄 Copying documentation...
copy *.md "%INSTALL_DIR%\" >nul 2>&1
if exist "docs" xcopy /E /I /Q docs "%INSTALL_DIR%\docs"

echo.
echo ✅ Nagari installed successfully!
echo.
echo ⚠️ Add to your PATH: %BIN_DIR%
echo 🔧 Verify: nag --version
pause
EOF
        else
            # Unix shell installer
            cat > "${PACKAGE_DIR}/install.sh" << 'EOF'
#!/bin/bash
set -e

echo "🚀 Installing Nagari Programming Language..."

INSTALL_DIR="${HOME}/.nagari"
BIN_DIR="${INSTALL_DIR}/bin"
STDLIB_DIR="${INSTALL_DIR}/stdlib"
RUNTIME_DIR="${INSTALL_DIR}/runtime"
EXAMPLES_DIR="${INSTALL_DIR}/examples"

mkdir -p "${BIN_DIR}" "${STDLIB_DIR}" "${RUNTIME_DIR}" "${EXAMPLES_DIR}"

echo "📁 Copying binaries..."
cp bin/* "${BIN_DIR}/"
chmod +x "${BIN_DIR}"/nag*

echo "📚 Copying standard library..."
if [ -d "stdlib" ]; then
    cp -r stdlib/* "${STDLIB_DIR}/"
fi

echo "⚡ Copying runtime..."
if [ -d "runtime" ]; then
    cp -r runtime/* "${RUNTIME_DIR}/"
fi

echo "📖 Copying examples..."
if [ -d "examples" ]; then
    cp -r examples/* "${EXAMPLES_DIR}/"
fi

echo "📄 Copying documentation..."
cp *.md "${INSTALL_DIR}/" 2>/dev/null || true
if [ -d "docs" ]; then
    cp -r docs "${INSTALL_DIR}/"
fi

echo ""
echo "✅ Nagari installed successfully!"
echo ""
echo "Add to your shell profile:"
echo "export PATH=\"\$HOME/.nagari/bin:\$PATH\""
echo ""
echo "Verify installation:"
echo "nag --version"
EOF
            chmod +x "${PACKAGE_DIR}/install.sh"
        fi

        # Create package-specific README
        cat > "${PACKAGE_DIR}/README.md" << EOF
# Nagari Programming Language v${VERSION}

**Target:** ${target}

This is a standalone executable package of the Nagari Programming Language for ${target}.

## Installation

EOF

        if [[ "$target" == *"windows"* ]]; then
            echo "Run \`install.bat\` to install Nagari to your system." >> "${PACKAGE_DIR}/README.md"
        else
            echo "Run \`./install.sh\` to install Nagari to your system." >> "${PACKAGE_DIR}/README.md"
        fi

        cat >> "${PACKAGE_DIR}/README.md" << EOF

## Quick Start

1. Try an example:
   \`nag run examples/hello.nag\`

2. Create and run a simple program:
   \`\`\`python
   # test.nag
   print("Hello from Nagari!")
   \`\`\`
   \`nag run test.nag\`

3. Compile to JavaScript:
   \`nag build test.nag\`

## Documentation

See the \`docs/\` directory for complete documentation.

## Support

- GitHub: https://github.com/ayanalamMOON/Nagari
- Issues: https://github.com/ayanalamMOON/Nagari/issues

---
Built for ${target} on $(date -u '+%Y-%m-%d')
EOF

        # Create archive
        cd packages/cross-platform

        if [[ "$target" == *"windows"* ]]; then
            ARCHIVE_NAME="${PACKAGE_NAME}.zip"
            if command -v zip >/dev/null 2>&1; then
                zip -r "${ARCHIVE_NAME}" "${PACKAGE_NAME}/"
            else
                print_warning "zip command not found for ${target}, creating tar.gz"
                tar -czf "${PACKAGE_NAME}.tar.gz" "${PACKAGE_NAME}/"
            fi
        else
            ARCHIVE_NAME="${PACKAGE_NAME}.tar.gz"
            tar -czf "${ARCHIVE_NAME}" "${PACKAGE_NAME}/"
        fi

        # Generate checksum
        if command -v sha256sum >/dev/null 2>&1; then
            sha256sum "${ARCHIVE_NAME}" > "${ARCHIVE_NAME}.sha256"
        elif command -v shasum >/dev/null 2>&1; then
            shasum -a 256 "${ARCHIVE_NAME}" > "${ARCHIVE_NAME}.sha256"
        fi

        cd "${ROOT_DIR}"

        SUCCESSFUL_BUILDS+=("${target}")
        print_success "Package created for ${target}: ${ARCHIVE_NAME}"

    else
        print_error "Build failed for ${target}"
        FAILED_BUILDS+=("${target}")
    fi

    echo ""
done

# Create distribution summary
print_step "Creating distribution summary"

cat > "packages/cross-platform/DISTRIBUTION.md" << EOF
# Nagari v${VERSION} - Cross-Platform Distribution

Generated on: $(date -u '+%Y-%m-%d %H:%M:%S UTC')

## Successful Builds

EOF

for target in "${SUCCESSFUL_BUILDS[@]}"; do
    echo "- **${target}**" >> "packages/cross-platform/DISTRIBUTION.md"
    ARCHIVE_NAME="nagari-${VERSION}-${target}"
    if [[ "$target" == *"windows"* ]]; then
        ARCHIVE_NAME="${ARCHIVE_NAME}.zip"
    else
        ARCHIVE_NAME="${ARCHIVE_NAME}.tar.gz"
    fi

    if [ -f "packages/cross-platform/${ARCHIVE_NAME}" ]; then
        SIZE=$(du -sh "packages/cross-platform/${ARCHIVE_NAME}" | cut -f1)
        echo "  - Archive: \`${ARCHIVE_NAME}\` (${SIZE})" >> "packages/cross-platform/DISTRIBUTION.md"

        if [ -f "packages/cross-platform/${ARCHIVE_NAME}.sha256" ]; then
            CHECKSUM=$(cut -d' ' -f1 "packages/cross-platform/${ARCHIVE_NAME}.sha256")
            echo "  - SHA256: \`${CHECKSUM}\`" >> "packages/cross-platform/DISTRIBUTION.md"
        fi
    fi
    echo "" >> "packages/cross-platform/DISTRIBUTION.md"
done

if [ ${#FAILED_BUILDS[@]} -gt 0 ]; then
    echo "## Failed Builds" >> "packages/cross-platform/DISTRIBUTION.md"
    echo "" >> "packages/cross-platform/DISTRIBUTION.md"
    for target in "${FAILED_BUILDS[@]}"; do
        echo "- ${target}" >> "packages/cross-platform/DISTRIBUTION.md"
    done
    echo "" >> "packages/cross-platform/DISTRIBUTION.md"
fi

cat >> "packages/cross-platform/DISTRIBUTION.md" << EOF
## Installation Instructions

### For End Users

1. Download the appropriate archive for your platform
2. Extract the archive to your preferred location
3. Run the installation script:
   - Windows: \`install.bat\`
   - Unix/Linux/macOS: \`./install.sh\`
4. Add the installation directory to your PATH if not done automatically
5. Verify: \`nag --version\`

### Platform Support

- **Linux x64**: \`nagari-${VERSION}-x86_64-unknown-linux-gnu.tar.gz\`
- **Windows x64**: \`nagari-${VERSION}-x86_64-pc-windows-msvc.zip\`
- **macOS Intel**: \`nagari-${VERSION}-x86_64-apple-darwin.tar.gz\`
- **macOS Apple Silicon**: \`nagari-${VERSION}-aarch64-apple-darwin.tar.gz\`
- **Linux ARM64**: \`nagari-${VERSION}-aarch64-unknown-linux-gnu.tar.gz\`

### System Requirements

- Operating System: Platform-specific
- Node.js: v16+ (for runtime features)
- Memory: 512MB RAM minimum
- Disk Space: ~50MB per installation

### Verification

All archives include SHA256 checksums for integrity verification.

## Support

- Documentation: https://github.com/ayanalamMOON/Nagari/docs
- Issues: https://github.com/ayanalamMOON/Nagari/issues
- Discussions: https://github.com/ayanalamMOON/Nagari/discussions

EOF

# Final summary
print_package "Cross-platform packaging completed!"
echo ""
echo "📊 Build Summary:"
echo "   ✅ Successful: ${#SUCCESSFUL_BUILDS[@]} targets"
echo "   ❌ Failed: ${#FAILED_BUILDS[@]} targets"
echo ""

if [ ${#SUCCESSFUL_BUILDS[@]} -gt 0 ]; then
    echo "📦 Successful packages:"
    for target in "${SUCCESSFUL_BUILDS[@]}"; do
        echo "   • ${target}"
    done
    echo ""
fi

if [ ${#FAILED_BUILDS[@]} -gt 0 ]; then
    echo "⚠️ Failed targets:"
    for target in "${FAILED_BUILDS[@]}"; do
        echo "   • ${target}"
    done
    echo ""
fi

echo "📁 Distribution directory: packages/cross-platform/"
echo "📋 Summary report: packages/cross-platform/DISTRIBUTION.md"
echo ""
echo "🚀 Ready for distribution!"
echo "   • Upload archives to GitHub Releases"
echo "   • Share download links with users"
echo "   • No compilation required on target systems"

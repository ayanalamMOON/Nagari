#!/bin/bash
# Complete package builder for Nagari Programming Language
# Creates standalone executable packages for external distribution
# Usage: ./scripts/package-release.sh [version] [target]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

print_step() { echo -e "${BLUE}🔷 $1${NC}"; }
print_success() { echo -e "${GREEN}✅ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠️ $1${NC}"; }
print_error() { echo -e "${RED}❌ $1${NC}"; }
print_info() { echo -e "${CYAN}ℹ️ $1${NC}"; }
print_package() { echo -e "${PURPLE}📦 $1${NC}"; }

# Configuration
VERSION=${1:-"0.3.0"}
TARGET=${2:-""}
ROOT_DIR=$(pwd)

# Auto-detect target if not provided
if [ -z "$TARGET" ]; then
    case "$(uname -s)" in
        Darwin)
            if [[ "$(uname -m)" == "arm64" ]]; then
                TARGET="aarch64-apple-darwin"
            else
                TARGET="x86_64-apple-darwin"
            fi
            ;;
        Linux)
            TARGET="x86_64-unknown-linux-gnu"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            TARGET="x86_64-pc-windows-msvc"
            ;;
        *)
            print_error "Unsupported platform: $(uname -s)"
            exit 1
            ;;
    esac
fi

print_step "Creating Nagari v${VERSION} executable package for ${TARGET}"

# Validate environment
if [ ! -f "Cargo.toml" ] || [ ! -d "src" ]; then
    print_error "This script must be run from the project root directory"
    exit 1
fi

# Check required tools
command -v rustc >/dev/null 2>&1 || { print_error "Rust is required but not installed"; exit 1; }
command -v cargo >/dev/null 2>&1 || { print_error "Cargo is required but not installed"; exit 1; }
command -v npm >/dev/null 2>&1 || { print_error "Node.js/npm is required but not installed"; exit 1; }

# Package configuration
PACKAGE_NAME="nagari-${VERSION}-${TARGET}"
PACKAGE_DIR="packages/${PACKAGE_NAME}"
ARCHIVE_NAME="${PACKAGE_NAME}.tar.gz"
if [[ "$TARGET" == *"windows"* ]]; then
    ARCHIVE_NAME="${PACKAGE_NAME}.zip"
fi

print_info "Package: ${PACKAGE_NAME}"
print_info "Archive: ${ARCHIVE_NAME}"

# Clean and create package directory
rm -rf "packages/${PACKAGE_NAME}"
mkdir -p "${PACKAGE_DIR}"/{bin,nagari-runtime,stdlib,examples,docs}

# Install target if needed
print_step "Ensuring Rust target ${TARGET} is available"
rustup target add ${TARGET} || print_warning "Target ${TARGET} may not be available"

# Build runtime first
print_step "Building Nagari runtime"
cd nagari-runtime

if [ ! -d "node_modules" ]; then
    npm install
fi

npm run build
print_success "Runtime built successfully"
cd "${ROOT_DIR}"

# Build Rust workspace for the target
print_step "Building Rust workspace for ${TARGET}"
cargo build --release --target ${TARGET} --workspace

print_success "Rust workspace built successfully"

# Determine binary extensions
CLI_BINARY="target/${TARGET}/release/nag"
LSP_BINARY="target/${TARGET}/release/nagari-lsp"
COMPILER_BINARY="target/${TARGET}/release/nagc"

if [[ "$TARGET" == *"windows"* ]]; then
    CLI_BINARY="${CLI_BINARY}.exe"
    LSP_BINARY="${LSP_BINARY}.exe"
    COMPILER_BINARY="${COMPILER_BINARY}.exe"
fi

# Verify binaries exist
print_step "Verifying built binaries"
for binary in "$CLI_BINARY" "$LSP_BINARY"; do
    if [ ! -f "$binary" ]; then
        print_error "Binary not found: $binary"
        exit 1
    fi
    print_success "Found: $(basename $binary)"
done

# Test functionality
print_step "Testing binary functionality"
$CLI_BINARY --version
$CLI_BINARY --help >/dev/null

# Quick compile test
echo 'print("Package build test successful!")' > test_package.nag
$CLI_BINARY build test_package.nag -o test_package.js
if [ -f "test_package.js" ] || [ -d "test_package.js" ] || [ -f "test_package.js/test_package.js" ]; then
    print_success "Compilation test passed"
    rm -f test_package.nag
    rm -rf test_package.js
else
    print_error "Compilation test failed"
    rm -f test_package.nag
    rm -rf test_package.js
    exit 1
fi

# Copy binaries
print_step "Packaging binaries"
cp "$CLI_BINARY" "${PACKAGE_DIR}/bin/"
cp "$LSP_BINARY" "${PACKAGE_DIR}/bin/"
if [ -f "$COMPILER_BINARY" ]; then
    cp "$COMPILER_BINARY" "${PACKAGE_DIR}/bin/"
fi

# Copy runtime
print_step "Packaging runtime"
mkdir -p "${PACKAGE_DIR}/nagari-runtime/dist"
cp -r nagari-runtime/dist/* "${PACKAGE_DIR}/nagari-runtime/dist/"
cp nagari-runtime/package.json "${PACKAGE_DIR}/nagari-runtime/"
cp nagari-runtime/README.md "${PACKAGE_DIR}/nagari-runtime/"

# Copy standard library
print_step "Packaging standard library"
if [ -d "stdlib" ]; then
    cp -r stdlib/* "${PACKAGE_DIR}/stdlib/"
fi

# Copy examples
print_step "Packaging examples"
if [ -d "examples" ]; then
    cp -r examples/* "${PACKAGE_DIR}/examples/"
fi

# Copy documentation
print_step "Packaging documentation"
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

# Create installation scripts
print_step "Creating installation scripts"

if [[ "$TARGET" != *"windows"* ]]; then
    # Unix installation script
    cat > "${PACKAGE_DIR}/install.sh" << 'INSTALL_EOF'
#!/bin/bash
set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 Installing Nagari Programming Language...${NC}"

INSTALL_DIR="${HOME}/.nagari"
BIN_DIR="${INSTALL_DIR}/bin"
STDLIB_DIR="${INSTALL_DIR}/stdlib"
EXAMPLES_DIR="${INSTALL_DIR}/examples"

mkdir -p "${BIN_DIR}" "${STDLIB_DIR}" "${EXAMPLES_DIR}"

echo "📁 Copying binaries..."
cp bin/* "${BIN_DIR}/"
chmod +x "${BIN_DIR}"/nag*

echo "📚 Copying standard library..."
if [ -d "stdlib" ]; then
    cp -r stdlib/* "${STDLIB_DIR}/"
fi

echo "⚡ Copying runtime..."
if [ -d "nagari-runtime" ]; then
    mkdir -p "${INSTALL_DIR}/nagari-runtime"
    cp -r nagari-runtime/* "${INSTALL_DIR}/nagari-runtime/"
    # Ensure runtime has correct structure
    if [ ! -f "${INSTALL_DIR}/nagari-runtime/package.json" ]; then
        echo "Warning: package.json not found in runtime"
    fi
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
echo -e "${GREEN}✅ Nagari installed successfully!${NC}"
echo ""
echo -e "${YELLOW}Add to your shell profile:${NC}"
echo "export PATH=\"\$HOME/.nagari/bin:\$PATH\""
echo ""
echo "Then restart your shell or run: source ~/.bashrc"
echo ""
echo "Verify: nag --version"
INSTALL_EOF
    chmod +x "${PACKAGE_DIR}/install.sh"

    # Unix uninstall script
    cat > "${PACKAGE_DIR}/uninstall.sh" << 'UNINSTALL_EOF'
#!/bin/bash
echo "🗑️ Uninstalling Nagari..."

INSTALL_DIR="${HOME}/.nagari"

if [ -d "${INSTALL_DIR}" ]; then
    rm -rf "${INSTALL_DIR}"
    echo "✅ Nagari uninstalled successfully"
else
    echo "ℹ️ Nagari is not installed"
fi
UNINSTALL_EOF
    chmod +x "${PACKAGE_DIR}/uninstall.sh"

else
    # Windows installation script
    cat > "${PACKAGE_DIR}/install.bat" << 'INSTALL_BAT_EOF'
@echo off
echo Installing Nagari Programming Language...

set "INSTALL_DIR=%USERPROFILE%\.nagari"
set "BIN_DIR=%INSTALL_DIR%\bin"
set "STDLIB_DIR=%INSTALL_DIR%\stdlib"
set "EXAMPLES_DIR=%INSTALL_DIR%\examples"

if not exist "%BIN_DIR%" mkdir "%BIN_DIR%"
if not exist "%STDLIB_DIR%" mkdir "%STDLIB_DIR%"
if not exist "%EXAMPLES_DIR%" mkdir "%EXAMPLES_DIR%"

echo Copying binaries...
copy bin\*.exe "%BIN_DIR%\"

echo Copying standard library...
if exist "stdlib" xcopy /E /I /Q stdlib "%STDLIB_DIR%"

echo Copying runtime...
if exist "nagari-runtime" xcopy /E /I /Q nagari-runtime "%INSTALL_DIR%\nagari-runtime"

echo Copying examples...
if exist "examples" xcopy /E /I /Q examples "%EXAMPLES_DIR%"

echo Copying documentation...
copy *.md "%INSTALL_DIR%\" >nul 2>&1
if exist "docs" xcopy /E /I /Q docs "%INSTALL_DIR%\docs"

echo.
echo Nagari installed successfully!
echo.
echo Add to your PATH: %BIN_DIR%
echo Verify: nag --version
pause
INSTALL_BAT_EOF

    # Windows uninstall script
    cat > "${PACKAGE_DIR}/uninstall.bat" << 'UNINSTALL_BAT_EOF'
@echo off
echo Uninstalling Nagari...

set "INSTALL_DIR=%USERPROFILE%\.nagari"

if exist "%INSTALL_DIR%" (
    rmdir /S /Q "%INSTALL_DIR%"
    echo Nagari uninstalled successfully
) else (
    echo Nagari is not installed
)
pause
UNINSTALL_BAT_EOF
fi

# Create README
cat > "${PACKAGE_DIR}/README.md" << README_EOF
# Nagari Programming Language v${VERSION}

Standalone executable package for ${TARGET}

## Installation

Run the installation script:
- Unix/Linux/macOS: \`./install.sh\`
- Windows: \`install.bat\`

## Quick Start

\`\`\`bash
# Run an example
nag run examples/hello.nag

# Compile to JavaScript
nag build examples/hello.nag
\`\`\`

## Contents

- **bin/** - Executable binaries
- **nagari-runtime/** - JavaScript runtime
- **stdlib/** - Standard library
- **examples/** - Example programs
- **docs/** - Documentation

## Support

- GitHub: https://github.com/ayanalamMOON/Nagari
- Issues: https://github.com/ayanalamMOON/Nagari/issues
README_EOF

# Show package summary
print_step "Package contents summary"
echo "📁 Files:"
find "${PACKAGE_DIR}" -type f | wc -l
echo ""
echo "📊 Size breakdown:"
du -sh "${PACKAGE_DIR}"/* | sort -hr

# Create archive
print_step "Creating archive: ${ARCHIVE_NAME}"
cd packages

if [[ "$TARGET" == *"windows"* ]]; then
    if command -v zip >/dev/null 2>&1; then
        zip -r "${PACKAGE_NAME}.zip" "${PACKAGE_NAME}/"
        ARCHIVE_NAME="${PACKAGE_NAME}.zip"
    else
        print_warning "zip command not found, creating tar.gz instead"
        tar -czf "${ARCHIVE_NAME}" "${PACKAGE_NAME}/"
    fi
else
    tar -czf "${ARCHIVE_NAME}" "${PACKAGE_NAME}/"
fi

cd "${ROOT_DIR}"

# Generate checksums
print_step "Generating checksums"
cd packages
if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${ARCHIVE_NAME}" > "${ARCHIVE_NAME}.sha256"
elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${ARCHIVE_NAME}" > "${ARCHIVE_NAME}.sha256"
fi
cd "${ROOT_DIR}"

# Final verification
print_step "Final verification"
if [ -f "packages/${ARCHIVE_NAME}" ]; then
    print_success "Archive created successfully"
    print_info "Size: $(du -sh packages/${ARCHIVE_NAME} | cut -f1)"
    if [ -f "packages/${ARCHIVE_NAME}.sha256" ]; then
        print_info "SHA256: $(cat packages/${ARCHIVE_NAME}.sha256 | cut -d' ' -f1)"
    fi
else
    print_error "Archive creation failed"
    exit 1
fi

# Success summary
print_package "Package created successfully!"
echo ""
echo "📦 Package: packages/${PACKAGE_NAME}/"
echo "📁 Archive: packages/${ARCHIVE_NAME}"
if [ -f "packages/${ARCHIVE_NAME}.sha256" ]; then
    echo "🔐 Checksum: packages/${ARCHIVE_NAME}.sha256"
fi
echo ""
echo "🚀 To test the package:"
echo "   cd packages/${PACKAGE_NAME}"
if [[ "$TARGET" != *"windows"* ]]; then
    echo "   ./install.sh"
else
    echo "   install.bat"
fi
echo "   nag --version"
echo ""
echo "📋 Distribution ready!"

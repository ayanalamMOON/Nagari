#!/bin/bash
# Local build script for Nagari Programming Language
# Usage: ./scripts/build.sh [target]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}🔷 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Get target from argument or detect current platform
TARGET=${1:-""}
if [ -z "$TARGET" ]; then
    case "$(uname -s)" in
        Darwin) TARGET="x86_64-apple-darwin" ;;
        Linux) TARGET="x86_64-unknown-linux-gnu" ;;
        CYGWIN*|MINGW*|MSYS*) TARGET="x86_64-pc-windows-msvc" ;;
        *) 
            print_error "Unsupported platform: $(uname -s)"
            exit 1
            ;;
    esac
fi

print_step "Building Nagari for target: ${TARGET}"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "src" ]; then
    print_error "This script must be run from the project root directory"
    exit 1
fi

# Install target if not already installed
print_step "Ensuring Rust target ${TARGET} is installed"
rustup target add ${TARGET}

# Build nagari-runtime first
print_step "Building nagari-runtime (npm package)"
cd nagari-runtime
if [ ! -d "node_modules" ]; then
    npm install
fi
npm run build
print_success "Runtime built successfully"
cd ..

# Build Rust workspace
print_step "Building Rust workspace"
cargo build --release --target ${TARGET}
print_success "Rust workspace built successfully"

# Test the built binaries
print_step "Testing built binaries"

CLI_BINARY="target/${TARGET}/release/nag"
LSP_BINARY="target/${TARGET}/release/nagari-lsp"

# Add .exe extension for Windows
if [[ "$TARGET" == *"windows"* ]]; then
    CLI_BINARY="${CLI_BINARY}.exe"
    LSP_BINARY="${LSP_BINARY}.exe"
fi

# Check if binaries exist
if [ ! -f "$CLI_BINARY" ]; then
    print_error "CLI binary not found: $CLI_BINARY"
    exit 1
fi

if [ ! -f "$LSP_BINARY" ]; then
    print_error "LSP binary not found: $LSP_BINARY"
    exit 1
fi

# Test CLI
print_step "Testing CLI functionality"
$CLI_BINARY --version
$CLI_BINARY --help >/dev/null

# Create a simple test file
echo 'print("Build test successful!")' > test_build.nag
$CLI_BINARY compile test_build.nag
if [ -f "test_build.js" ]; then
    print_success "Compilation test passed"
    rm -f test_build.nag test_build.js
else
    print_error "Compilation test failed"
    exit 1
fi

# Test LSP (basic check)
print_step "Testing LSP server"
timeout 2s $LSP_BINARY --help >/dev/null 2>&1 || true
print_success "LSP server starts correctly"

# Create distribution directory
DIST_DIR="dist/nagari-${TARGET}"
print_step "Creating distribution package in ${DIST_DIR}"

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR/bin"
mkdir -p "$DIST_DIR/stdlib"
mkdir -p "$DIST_DIR/runtime"

# Copy binaries
cp "$CLI_BINARY" "$DIST_DIR/bin/"
cp "$LSP_BINARY" "$DIST_DIR/bin/"

# Copy documentation and licenses
cp README.md "$DIST_DIR/"
cp LICENSE "$DIST_DIR/"
if [ -f "CHANGELOG.md" ]; then
    cp CHANGELOG.md "$DIST_DIR/"
fi

# Copy standard library
if [ -d "stdlib" ]; then
    cp -r stdlib/* "$DIST_DIR/stdlib/"
fi

# Copy runtime
cp -r nagari-runtime/dist/* "$DIST_DIR/runtime/"
cp nagari-runtime/package.json "$DIST_DIR/runtime/"

# Create installation script for Unix-like systems
if [[ "$TARGET" != *"windows"* ]]; then
    cat > "$DIST_DIR/install.sh" << 'EOF'
#!/bin/bash
set -e

echo "🚀 Installing Nagari Programming Language..."

# Create installation directory
INSTALL_DIR="${HOME}/.nagari"
BIN_DIR="${INSTALL_DIR}/bin"
STDLIB_DIR="${INSTALL_DIR}/stdlib"
RUNTIME_DIR="${INSTALL_DIR}/runtime"

mkdir -p "${BIN_DIR}" "${STDLIB_DIR}" "${RUNTIME_DIR}"

# Copy binaries
cp bin/* "${BIN_DIR}/"
chmod +x "${BIN_DIR}"/nag*

# Copy standard library and runtime
if [ -d "stdlib" ]; then
    cp -r stdlib/* "${STDLIB_DIR}/"
fi
if [ -d "runtime" ]; then
    cp -r runtime/* "${RUNTIME_DIR}/"
fi

echo ""
echo "✅ Nagari installed successfully!"
echo ""
echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
echo "export PATH=\"\$HOME/.nagari/bin:\$PATH\""
echo ""
echo "Then restart your shell or run:"
echo "source ~/.bashrc  # or your shell's config file"
echo ""
echo "Verify installation:"
echo "nag --version"
EOF
    chmod +x "$DIST_DIR/install.sh"
else
    # Create Windows installation script
    cat > "$DIST_DIR/install.bat" << 'EOF'
@echo off
echo 🚀 Installing Nagari Programming Language...

REM Create installation directory
set "INSTALL_DIR=%USERPROFILE%\.nagari"
set "BIN_DIR=%INSTALL_DIR%\bin"
set "STDLIB_DIR=%INSTALL_DIR%\stdlib"
set "RUNTIME_DIR=%INSTALL_DIR%\runtime"

if not exist "%BIN_DIR%" mkdir "%BIN_DIR%"
if not exist "%STDLIB_DIR%" mkdir "%STDLIB_DIR%"
if not exist "%RUNTIME_DIR%" mkdir "%RUNTIME_DIR%"

REM Copy binaries
copy bin\*.exe "%BIN_DIR%\"

REM Copy standard library and runtime
if exist "stdlib" xcopy /E /I stdlib "%STDLIB_DIR%"
if exist "runtime" xcopy /E /I runtime "%RUNTIME_DIR%"

echo.
echo ✅ Nagari installed successfully!
echo.
echo Add the following directory to your PATH:
echo %BIN_DIR%
echo.
echo Verify installation:
echo nag --version
EOF
fi

# Show binary information
print_step "Build information"
echo "Target: ${TARGET}"
echo "CLI binary: $(ls -lh $CLI_BINARY | awk '{print $5}')"
echo "LSP binary: $(ls -lh $LSP_BINARY | awk '{print $5}')"
echo "Distribution: ${DIST_DIR}"

print_success "Build completed successfully!"
echo ""
echo "📦 Distribution package created in: ${DIST_DIR}"
echo "🚀 To install locally, run the installation script in the distribution directory"
echo "🔧 To test the build:"
echo "   cd ${DIST_DIR}"
if [[ "$TARGET" != *"windows"* ]]; then
    echo "   ./install.sh"
else
    echo "   install.bat"
fi
echo "   nag --version"

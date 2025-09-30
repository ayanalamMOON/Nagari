#!/bin/bash

# Nagari Development Environment Setup
# This script sets up a complete development environment for Nagari

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUST_VERSION="1.70.0"
NODE_VERSION="18.0.0"

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  Nagari Development Setup${NC}"
    echo -e "${BLUE}================================${NC}"
    echo
}

print_step() {
    echo -e "${YELLOW}[STEP]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

check_prerequisites() {
    print_step "Checking prerequisites..."

    # Check if running on supported platform
    case "$(uname -s)" in
        Linux*|Darwin*|MINGW*|MSYS*|CYGWIN*)
            print_success "Platform supported"
            ;;
        *)
            print_error "Unsupported platform: $(uname -s)"
            exit 1
            ;;
    esac

    # Check for required tools
    local missing_tools=()

    if ! command -v git &> /dev/null; then
        missing_tools+=("git")
    fi

    if ! command -v curl &> /dev/null; then
        missing_tools+=("curl")
    fi

    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_error "Missing required tools: ${missing_tools[*]}"
        print_info "Please install them and run this script again"
        exit 1
    fi

    print_success "Prerequisites check passed"
}

install_rust() {
    print_step "Setting up Rust development environment..."

    if command -v rustc &> /dev/null; then
        local current_version=$(rustc --version | cut -d' ' -f2)
        print_info "Rust already installed: $current_version"
    else
        print_info "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    # Update Rust components
    print_info "Updating Rust components..."
    rustup update
    rustup component add clippy rustfmt rust-analyzer

    # Install useful cargo tools
    print_info "Installing cargo tools..."
    cargo install --locked cargo-watch cargo-edit cargo-audit cargo-outdated || true

    print_success "Rust environment ready"
}

install_node() {
    print_step "Setting up Node.js environment..."

    if command -v node &> /dev/null; then
        local current_version=$(node --version)
        print_info "Node.js already installed: $current_version"
    else
        print_info "Please install Node.js $NODE_VERSION or later"
        print_info "Visit: https://nodejs.org/"

        # Check if we can install via package manager
        if command -v apt &> /dev/null; then
            print_info "You can install with: sudo apt install nodejs npm"
        elif command -v brew &> /dev/null; then
            print_info "You can install with: brew install node"
        elif command -v winget &> /dev/null; then
            print_info "You can install with: winget install OpenJS.NodeJS"
        fi

        read -p "Press Enter when Node.js is installed..."
    fi

    # Update npm
    if command -v npm &> /dev/null; then
        print_info "Updating npm..."
        npm install -g npm@latest || true
    fi

    print_success "Node.js environment ready"
}

setup_project() {
    print_step "Setting up project dependencies..."

    cd "$PROJECT_ROOT"

    # Install Rust dependencies
    print_info "Building Rust project..."
    cargo check

    # Setup TypeScript runtime
    if [ -d "nagari-runtime" ]; then
        print_info "Installing runtime dependencies..."
        cd nagari-runtime
        npm install
        npm run build
        cd "$PROJECT_ROOT"
    fi

    # Setup VS Code extension if present
    if [ -d "vscode-extension" ]; then
        print_info "Installing VS Code extension dependencies..."
        cd vscode-extension
        npm install
        cd "$PROJECT_ROOT"
    fi

    print_success "Project dependencies installed"
}

create_dev_config() {
    print_step "Creating development configuration..."

    cat > "$PROJECT_ROOT/.nagari-dev.json" << 'EOF'
{
  "environment": "development",
  "compiler": {
    "debug": true,
    "optimize": false,
    "emit_source_maps": true
  },
  "runtime": {
    "enable_debugging": true,
    "verbose_logging": true
  },
  "testing": {
    "parallel": true,
    "coverage": true,
    "watch_mode": true
  },
  "lsp": {
    "enable_diagnostics": true,
    "completion": true,
    "hover_info": true
  },
  "tools": {
    "auto_format": true,
    "lint_on_save": true,
    "pre_commit_hooks": true
  }
}
EOF

    print_success "Development configuration created"
}

setup_git_hooks() {
    print_step "Setting up Git hooks..."

    cd "$PROJECT_ROOT"

    # Create pre-commit hook
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Nagari pre-commit hook

echo "Running pre-commit checks..."

# Format code
cargo fmt --check
if [ $? -ne 0 ]; then
    echo "Code formatting check failed. Run 'cargo fmt' to fix."
    exit 1
fi

# Lint code
cargo clippy -- -D warnings
if [ $? -ne 0 ]; then
    echo "Linting failed. Fix clippy warnings before committing."
    exit 1
fi

# Run tests
cargo test
if [ $? -ne 0 ]; then
    echo "Tests failed. Fix failing tests before committing."
    exit 1
fi

echo "Pre-commit checks passed!"
EOF

    chmod +x .git/hooks/pre-commit

    print_success "Git hooks installed"
}

setup_ide() {
    print_step "Setting up IDE configuration..."

    # Create VS Code settings
    mkdir -p "$PROJECT_ROOT/.vscode"

    cat > "$PROJECT_ROOT/.vscode/settings.json" << 'EOF'
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.completion.addCallParentheses": false,
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
        "source.fixAll": true
    },
    "files.associations": {
        "*.nag": "python"
    },
    "typescript.preferences.includePackageJsonAutoImports": "on",
    "npm.enableScriptExplorer": true,
    "terminal.integrated.defaultProfile.windows": "Git Bash"
}
EOF

    cat > "$PROJECT_ROOT/.vscode/extensions.json" << 'EOF'
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "ms-vscode.vscode-typescript-next",
        "ms-python.python",
        "bradlc.vscode-tailwindcss",
        "GitHub.copilot",
        "ms-vscode.test-adapter-converter",
        "hbenl.vscode-test-explorer"
    ]
}
EOF

    print_success "IDE configuration created"
}

create_dev_scripts() {
    print_step "Creating development scripts..."

    # Create quick development shortcuts
    cat > "$PROJECT_ROOT/dev.sh" << 'EOF'
#!/bin/bash
# Quick development commands

case "$1" in
    "build")
        cargo build
        ;;
    "test")
        cargo test
        ;;
    "run")
        shift
        cargo run -- "$@"
        ;;
    "format")
        cargo fmt
        ;;
    "lint")
        cargo clippy
        ;;
    "clean")
        cargo clean
        rm -rf dist/
        ;;
    "watch")
        cargo watch -x check -x test
        ;;
    *)
        echo "Usage: $0 {build|test|run|format|lint|clean|watch}"
        echo ""
        echo "Commands:"
        echo "  build    - Build the project"
        echo "  test     - Run tests"
        echo "  run      - Run the CLI (pass arguments after 'run')"
        echo "  format   - Format code"
        echo "  lint     - Run linter"
        echo "  clean    - Clean build artifacts"
        echo "  watch    - Watch for changes and run checks"
        ;;
esac
EOF

    chmod +x "$PROJECT_ROOT/dev.sh"

    print_success "Development scripts created"
}

print_summary() {
    echo
    echo -e "${GREEN}================================${NC}"
    echo -e "${GREEN}  Setup Complete!${NC}"
    echo -e "${GREEN}================================${NC}"
    echo
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Run tests: ${BLUE}./dev.sh test${NC}"
    echo "2. Start development: ${BLUE}./dev.sh watch${NC}"
    echo "3. Build project: ${BLUE}./dev.sh build${NC}"
    echo "4. Open in VS Code and install recommended extensions"
    echo
    echo -e "${YELLOW}Available tools:${NC}"
    echo "• ${BLUE}./dev.sh${NC} - Quick development commands"
    echo "• ${BLUE}./dev-tools/${NC} - Complete development toolkit"
    echo "• ${BLUE}.nagari-dev.json${NC} - Development configuration"
    echo
    echo -e "${GREEN}Happy coding! 🚀${NC}"
}

main() {
    print_header
    check_prerequisites
    install_rust
    install_node
    setup_project
    create_dev_config
    setup_git_hooks
    setup_ide
    create_dev_scripts
    print_summary
}

# Help text
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Nagari Development Environment Setup"
    echo
    echo "This script sets up a complete development environment for Nagari including:"
    echo "• Rust toolchain with necessary components"
    echo "• Node.js environment for runtime development"
    echo "• Project dependencies and build tools"
    echo "• Git hooks for code quality"
    echo "• IDE configuration for VS Code"
    echo "• Development shortcuts and tools"
    echo
    echo "Usage: $0 [--help]"
    echo
    echo "The script will guide you through the setup process and create all"
    echo "necessary configuration files for productive development."
    exit 0
fi

main "$@"

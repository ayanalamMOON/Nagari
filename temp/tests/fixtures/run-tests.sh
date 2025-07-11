#!/bin/bash

# Comprehensive test runner for Nagari ecosystem
set -e

echo "🧪 Running Nagari Ecosystem Tests"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        return 1
    fi
}

# Function to run tests in a directory
run_tests() {
    local dir=$1
    local name=$2

    echo ""
    echo -e "${YELLOW}Testing $name...${NC}"

    if [ -d "$dir" ]; then
        cd "$dir"

        # Check if Cargo.toml exists
        if [ -f "Cargo.toml" ]; then
            # Run unit tests
            echo "Running unit tests..."
            cargo test --lib
            print_status $? "Unit tests for $name"

            # Run integration tests
            if [ -d "tests" ]; then
                echo "Running integration tests..."
                cargo test --test '*'
                print_status $? "Integration tests for $name"
            fi

            # Run doctests
            echo "Running doc tests..."
            cargo test --doc
            print_status $? "Doc tests for $name"

            # Check formatting
            echo "Checking code formatting..."
            cargo fmt -- --check
            print_status $? "Code formatting for $name"

            # Run clippy
            echo "Running clippy..."
            cargo clippy -- -D warnings
            print_status $? "Clippy checks for $name"

        else
            echo "No Cargo.toml found in $dir, skipping..."
        fi

        cd - > /dev/null
    else
        echo "Directory $dir not found, skipping..."
    fi
}

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

echo "Project root: $PROJECT_ROOT"

# Test each component
run_tests "cli" "CLI Tool"
run_tests "registry-server" "Registry Server"
run_tests "lsp-server" "LSP Server"

# Run end-to-end tests
echo ""
echo -e "${YELLOW}Running End-to-End Tests...${NC}"

# Test CLI installation and basic functionality
echo "Testing CLI installation..."
cd cli
cargo build --release
print_status $? "CLI build"

# Test basic CLI commands
echo "Testing basic CLI commands..."
./target/release/nag --help > /dev/null
print_status $? "CLI help command"

# Test package manager commands
echo "Testing package manager..."
cd tests/fixtures
if [ ! -d "test-project" ]; then
    mkdir -p test-project
    cd test-project

    # Initialize test project
    ../../../target/release/nag package init --yes
    print_status $? "Package initialization"

    # Test package list
    ../../../target/release/nag package list
    print_status $? "Package list"

    cd ..
fi

cd "$PROJECT_ROOT"

# Test registry server
echo ""
echo "Testing registry server..."
cd registry-server

# Build registry server
cargo build --release
print_status $? "Registry server build"

# Start registry server in background for testing
echo "Starting registry server for testing..."
REGISTRY_PID=""
if command -v timeout >/dev/null 2>&1; then
    timeout 30s ./target/release/nagari-registry --port 3001 &
    REGISTRY_PID=$!
    sleep 2

    # Test health endpoint
    if command -v curl >/dev/null 2>&1; then
        curl -f http://localhost:3001/health > /dev/null 2>&1
        print_status $? "Registry health check"

        # Test API documentation
        curl -f http://localhost:3001/docs > /dev/null 2>&1
        print_status $? "Registry API docs"
    else
        echo "curl not available, skipping HTTP tests"
    fi

    # Kill registry server
    if [ ! -z "$REGISTRY_PID" ]; then
        kill $REGISTRY_PID 2>/dev/null || true
    fi
else
    echo "timeout command not available, skipping registry server runtime tests"
fi

cd "$PROJECT_ROOT"

# Test LSP server
echo ""
echo "Testing LSP server..."
cd lsp-server

# Build LSP server
cargo build --release
print_status $? "LSP server build"

# Test LSP server help
./target/release/nagari-lsp --help > /dev/null
print_status $? "LSP server help"

cd "$PROJECT_ROOT"

# Performance tests
echo ""
echo -e "${YELLOW}Running Performance Tests...${NC}"

# Test compilation performance
echo "Testing compilation performance..."
cd cli
time cargo build --release > /dev/null 2>&1
print_status $? "Compilation performance"

cd "$PROJECT_ROOT"

# Code coverage (if available)
if command -v cargo-tarpaulin >/dev/null 2>&1; then
    echo ""
    echo -e "${YELLOW}Generating Code Coverage...${NC}"

    cd cli
    cargo tarpaulin --out Html --output-dir ../coverage
    print_status $? "Code coverage generation"

    cd "$PROJECT_ROOT"
    echo "Coverage report generated in coverage/tarpaulin-report.html"
else
    echo ""
    echo "cargo-tarpaulin not available, skipping coverage report"
    echo "Install with: cargo install cargo-tarpaulin"
fi

# Security audit (if available)
if command -v cargo-audit >/dev/null 2>&1; then
    echo ""
    echo -e "${YELLOW}Running Security Audit...${NC}"

    cd cli
    cargo audit
    print_status $? "Security audit for CLI"

    cd "$PROJECT_ROOT/registry-server"
    cargo audit
    print_status $? "Security audit for Registry"

    cd "$PROJECT_ROOT/lsp-server"
    cargo audit
    print_status $? "Security audit for LSP"

    cd "$PROJECT_ROOT"
else
    echo ""
    echo "cargo-audit not available, skipping security audit"
    echo "Install with: cargo install cargo-audit"
fi

echo ""
echo -e "${GREEN}🎉 All tests completed!${NC}"
echo ""
echo "Summary:"
echo "- CLI Tool: Unit, integration, and performance tests"
echo "- Registry Server: API and functionality tests"
echo "- LSP Server: Language server capabilities"
echo "- Code formatting and linting checks"
echo "- Security audits (if available)"
echo ""
echo "Next steps:"
echo "1. Review any failed tests above"
echo "2. Check coverage report (if generated)"
echo "3. Run manual integration tests"
echo "4. Test in different environments"

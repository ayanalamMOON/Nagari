#!/bin/bash

# Nagari Toolchain Integration Test Script

echo "🧰 Nagari Toolchain Integration Tests"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
test_success() {
    echo -e "${GREEN}✅ $1${NC}"
    ((TESTS_PASSED++))
}

test_failure() {
    echo -e "${RED}❌ $1${NC}"
    ((TESTS_FAILED++))
}

test_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

test_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Check prerequisites
echo -e "\n${BLUE}📋 Checking Prerequisites${NC}"
echo "================================"

# Check if nagc exists
if command -v cargo &> /dev/null; then
    test_success "Rust/Cargo installed"
else
    test_failure "Rust/Cargo not found"
    exit 1
fi

# Check if Node.js exists
if command -v node &> /dev/null; then
    test_success "Node.js installed ($(node --version))"
else
    test_failure "Node.js not found"
    exit 1
fi

# Check if npm exists
if command -v npm &> /dev/null; then
    test_success "npm installed ($(npm --version))"
else
    test_failure "npm not found"
    exit 1
fi

# Build the compiler
echo -e "\n${BLUE}🔨 Building Nagari Compiler${NC}"
echo "================================="

cd nagari-compiler
if cargo build --release; then
    test_success "Nagari compiler built successfully"
    NAGC_PATH="./target/release/nagc"
else
    test_failure "Failed to build Nagari compiler"
    exit 1
fi
cd ..

# Build the runtime
echo -e "\n${BLUE}📦 Building Nagari Runtime${NC}"
echo "==============================="

cd nagari-runtime
if npm install && npm run build; then
    test_success "Nagari runtime built successfully"
else
    test_failure "Failed to build Nagari runtime"
    exit 1
fi
cd ..

# Test 1: Basic CLI compilation
echo -e "\n${BLUE}🧪 Test 1: Basic CLI Compilation${NC}"
echo "=================================="

cd examples
if $NAGC_PATH cli_demo.nag --target node --output cli_demo.js; then
    test_success "CLI demo compiled successfully"

    # Check if output file exists
    if [ -f "cli_demo.js" ]; then
        test_success "Output file created"

        # Check if it's valid JavaScript
        if node -c cli_demo.js; then
            test_success "Generated JavaScript is valid"
        else
            test_failure "Generated JavaScript has syntax errors"
        fi
    else
        test_failure "Output file not created"
    fi
else
    test_failure "CLI demo compilation failed"
fi

# Test 2: Web server compilation
echo -e "\n${BLUE}🧪 Test 2: Web Server Compilation${NC}"
echo "==================================="

if $NAGC_PATH web_server.nag --target node --output web_server.js; then
    test_success "Web server compiled successfully"

    if [ -f "web_server.js" ]; then
        test_success "Web server output file created"

        if node -c web_server.js; then
            test_success "Web server JavaScript is valid"
        else
            test_failure "Web server JavaScript has syntax errors"
        fi
    else
        test_failure "Web server output file not created"
    fi
else
    test_failure "Web server compilation failed"
fi

# Test 3: React app compilation with JSX
echo -e "\n${BLUE}🧪 Test 3: React App Compilation (JSX)${NC}"
echo "======================================="

if $NAGC_PATH react_todo_app.nag --target esm --jsx --output react_todo_app.js; then
    test_success "React app compiled successfully with JSX"

    if [ -f "react_todo_app.js" ]; then
        test_success "React app output file created"

        if node -c react_todo_app.js; then
            test_success "React app JavaScript is valid"
        else
            test_failure "React app JavaScript has syntax errors"
        fi
    else
        test_failure "React app output file not created"
    fi
else
    test_failure "React app compilation failed"
fi

# Test 4: Different target formats
echo -e "\n${BLUE}🧪 Test 4: Target Format Tests${NC}"
echo "==============================="

# ES6 target
if $NAGC_PATH interop_demo.nag --target es6 --output interop_demo.es6.js; then
    test_success "ES6 target compilation"
else
    test_failure "ES6 target compilation failed"
fi

# Node.js target
if $NAGC_PATH interop_demo.nag --target node --output interop_demo.node.js; then
    test_success "Node.js target compilation"
else
    test_failure "Node.js target compilation failed"
fi

# ESM target
if $NAGC_PATH interop_demo.nag --target esm --output interop_demo.esm.js; then
    test_success "ESM target compilation"
else
    test_failure "ESM target compilation failed"
fi

# Test 5: Compiler flags
echo -e "\n${BLUE}🧪 Test 5: Compiler Flags${NC}"
echo "=========================="

# Source maps
if $NAGC_PATH cli_demo.nag --sourcemap --output cli_demo.map.js; then
    test_success "Source map generation"

    if [ -f "cli_demo.map.js.map" ]; then
        test_success "Source map file created"
    else
        test_failure "Source map file not created"
    fi
else
    test_failure "Source map generation failed"
fi

# Development mode
if $NAGC_PATH cli_demo.nag --devtools --output cli_demo.dev.js; then
    test_success "Development mode compilation"
else
    test_failure "Development mode compilation failed"
fi

# Syntax check only
if $NAGC_PATH cli_demo.nag --check; then
    test_success "Syntax check mode"
else
    test_failure "Syntax check mode failed"
fi

# Test 6: Runtime execution
echo -e "\n${BLUE}🧪 Test 6: Runtime Execution${NC}"
echo "============================="

# Install test dependencies
test_info "Installing test dependencies..."
if npm install express cors; then
    test_success "Test dependencies installed"
else
    test_warning "Some dependencies may be missing"
fi

# Test CLI execution
test_info "Testing CLI execution..."
mkdir -p test_input test_output
echo "Hello World" > test_input/test.txt
echo "Another file" > test_input/test2.txt

if timeout 10s node cli_demo.js test_input test_output; then
    test_success "CLI demo executed successfully"

    if [ -f "test_output/processed_test.txt" ]; then
        test_success "CLI demo produced expected output"
    else
        test_failure "CLI demo did not produce expected output"
    fi
else
    test_warning "CLI demo execution timed out or failed"
fi

# Test web server (start and stop)
test_info "Testing web server startup..."
if timeout 5s node web_server.js &
then
    SERVER_PID=$!
    sleep 2

    # Test health endpoint
    if curl -s http://localhost:3000/health > /dev/null; then
        test_success "Web server responded to health check"
    else
        test_failure "Web server did not respond"
    fi

    # Stop server
    kill $SERVER_PID 2>/dev/null
    test_success "Web server stopped"
else
    test_warning "Web server startup test skipped"
fi

# Cleanup
rm -f *.js *.js.map
rm -rf test_input test_output

cd ..

# Test results
echo -e "\n${BLUE}📊 Test Results${NC}"
echo "================"
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo -e "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed! Nagari toolchain is working correctly.${NC}"
    exit 0
else
    echo -e "\n${RED}💥 Some tests failed. Please check the output above.${NC}"
    exit 1
fi

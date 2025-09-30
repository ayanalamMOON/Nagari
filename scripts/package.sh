#!/bin/bash
# Nagari Package Builder - Simple Interface
# Usage: ./package.sh [command] [options]

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_usage() {
    echo "Nagari Package Builder"
    echo ""
    echo "Usage: ./package.sh [command] [options]"
    echo ""
    echo "Commands:"
    echo "  single [version] [target]    Build package for single platform"
    echo "  multi [version]              Build packages for all platforms"
    echo "  clean                        Clean package directory"
    echo "  test                         Test build and packaging"
    echo "  help                         Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./package.sh single                     # Current platform, default version"
    echo "  ./package.sh single 0.3.0               # Specific version, current platform"
    echo "  ./package.sh single 0.3.0 linux        # Specific version and platform"
    echo "  ./package.sh multi 0.3.0                # All platforms"
    echo "  ./package.sh test                       # Quick test build"
    echo ""
    echo "Platforms:"
    echo "  linux     - x86_64-unknown-linux-gnu"
    echo "  windows   - x86_64-pc-windows-msvc"
    echo "  macos     - x86_64-apple-darwin"
    echo "  macos-arm - aarch64-apple-darwin"
    echo "  linux-arm - aarch64-unknown-linux-gnu"
}

# Map friendly names to Rust targets
get_target() {
    case "$1" in
        linux) echo "x86_64-unknown-linux-gnu" ;;
        windows) echo "x86_64-pc-windows-msvc" ;;
        macos) echo "x86_64-apple-darwin" ;;
        macos-arm) echo "aarch64-apple-darwin" ;;
        linux-arm) echo "aarch64-unknown-linux-gnu" ;;
        *) echo "$1" ;;  # Pass through if already a valid target
    esac
}

# Get current platform target
get_current_target() {
    case "$(uname -s)" in
        Darwin)
            if [[ "$(uname -m)" == "arm64" ]]; then
                echo "aarch64-apple-darwin"
            else
                echo "x86_64-apple-darwin"
            fi
            ;;
        Linux) echo "x86_64-unknown-linux-gnu" ;;
        CYGWIN*|MINGW*|MSYS*) echo "x86_64-pc-windows-msvc" ;;
        *) echo "unknown" ;;
    esac
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "src" ]; then
    echo -e "${RED}❌ Error: This script must be run from the Nagari project root${NC}"
    exit 1
fi

COMMAND=${1:-help}
VERSION=${2:-"0.3.0"}
TARGET_INPUT=${3:-""}

case "$COMMAND" in
    "single")
        echo -e "${BLUE}🔷 Building single platform package${NC}"

        if [ -n "$TARGET_INPUT" ]; then
            TARGET=$(get_target "$TARGET_INPUT")
        else
            TARGET=$(get_current_target)
        fi

        echo "Version: $VERSION"
        echo "Target: $TARGET"
        echo ""

        if [ -f "scripts/package-release.sh" ]; then
            ./scripts/package-release.sh "$VERSION" "$TARGET"
        else
            echo -e "${RED}❌ Error: scripts/package-release.sh not found${NC}"
            exit 1
        fi
        ;;

    "multi")
        echo -e "${BLUE}🔷 Building multi-platform packages${NC}"
        echo "Version: $VERSION"
        echo ""

        if [ -f "scripts/package-cross-platform.sh" ]; then
            ./scripts/package-cross-platform.sh "$VERSION"
        else
            echo -e "${RED}❌ Error: scripts/package-cross-platform.sh not found${NC}"
            exit 1
        fi
        ;;

    "clean")
        echo -e "${BLUE}🔷 Cleaning package directory${NC}"
        if [ -d "packages" ]; then
            rm -rf packages/*
            echo -e "${GREEN}✅ Packages directory cleaned${NC}"
        else
            echo -e "${YELLOW}⚠️ No packages directory found${NC}"
        fi
        ;;

    "test")
        echo -e "${BLUE}🔷 Running test build${NC}"

        # Quick test: build CLI only
        echo "Building CLI for current platform..."
        if cargo build --release --bin nag; then
            echo -e "${GREEN}✅ CLI build successful${NC}"

            # Test basic functionality
            echo "Testing CLI functionality..."
            if ./target/release/nag --version >/dev/null 2>&1; then
                echo -e "${GREEN}✅ CLI test successful${NC}"

                # Test compilation
                echo 'print("Test successful!")' > test_package_build.nag
                if ./target/release/nag build test_package_build.nag >/dev/null 2>&1; then
                    echo -e "${GREEN}✅ Compilation test successful${NC}"
                    rm -f test_package_build.nag
                    rm -rf test_package_build.js
                else
                    echo -e "${YELLOW}⚠️ Compilation test failed${NC}"
                    rm -f test_package_build.nag
                    rm -rf test_package_build.js
                fi
            else
                echo -e "${RED}❌ CLI test failed${NC}"
                exit 1
            fi
        else
            echo -e "${RED}❌ CLI build failed${NC}"
            exit 1
        fi
        ;;

    "help"|*)
        print_usage
        ;;
esac

#!/bin/bash
# Nagari Development Master Launcher
# Entry point for all development tasks

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="$SCRIPT_DIR/config.json"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ASCII Art Banner
show_banner() {
    echo -e "${PURPLE}"
    echo "╔═══════════════════════════════════════════════════════════════════════════════╗"
    echo "║                            🚀 NAGARI DEV TOOLS 🚀                            ║"
    echo "║                     Comprehensive Development Toolkit                        ║"
    echo "╚═══════════════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Show usage information
show_usage() {
    echo -e "${CYAN}Usage: $0 [command] [options]${NC}"
    echo
    echo -e "${YELLOW}Available Commands:${NC}"
    echo -e "  ${GREEN}setup${NC}      - Setup development environment"
    echo -e "  ${GREEN}server${NC}     - Start development server with hot reload"
    echo -e "  ${GREEN}test${NC}       - Run comprehensive test suite"
    echo -e "  ${GREEN}lint${NC}       - Run code quality checks and formatting"
    echo -e "  ${GREEN}version${NC}    - Bump version (major|minor|patch)"
    echo -e "  ${GREEN}release${NC}    - Prepare release packages"
    echo -e "  ${GREEN}build${NC}      - Build project (debug|release)"
    echo -e "  ${GREEN}clean${NC}      - Clean build artifacts"
    echo -e "  ${GREEN}watch${NC}      - Watch for changes and rebuild"
    echo -e "  ${GREEN}status${NC}     - Show project status"
    echo -e "  ${GREEN}help${NC}       - Show this help message"
    echo
    echo -e "${YELLOW}Quick Commands:${NC}"
    echo -e "  ${BLUE}dev${NC}        - Equivalent to: setup && server"
    echo -e "  ${BLUE}check${NC}      - Equivalent to: lint && test"
    echo -e "  ${BLUE}ship${NC}       - Equivalent to: lint && test && release"
    echo
    echo -e "${YELLOW}Examples:${NC}"
    echo -e "  $0 setup                    # Setup development environment"
    echo -e "  $0 server --port 4000       # Start server on port 4000"
    echo -e "  $0 test --coverage          # Run tests with coverage"
    echo -e "  $0 version minor            # Bump minor version"
    echo -e "  $0 release --all-platforms  # Build for all platforms"
    echo
}

# Check if running on Windows (Git Bash/WSL)
is_windows() {
    [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ -n "$WSL_DISTRO_NAME" ]]
}

# Get the correct script extension
get_script_ext() {
    if is_windows; then
        echo ".bat"
    else
        echo ".sh"
    fi
}

# Execute a development tool
run_tool() {
    local tool_name="$1"
    shift
    local script_ext=$(get_script_ext)
    local script_path="$SCRIPT_DIR/${tool_name}${script_ext}"

    if [[ ! -f "$script_path" ]]; then
        echo -e "${RED}Error: Tool '$tool_name' not found at $script_path${NC}"
        return 1
    fi

    echo -e "${BLUE}Running: $tool_name${NC}"

    if [[ "$script_ext" == ".bat" ]]; then
        cmd.exe /c "$script_path" "$@"
    else
        chmod +x "$script_path" 2>/dev/null || true
        "$script_path" "$@"
    fi
}

# Show project status
show_status() {
    echo -e "${CYAN}=== Nagari Project Status ===${NC}"
    echo

    # Git status
    if command -v git >/dev/null 2>&1; then
        echo -e "${YELLOW}Git Status:${NC}"
        cd "$PROJECT_ROOT"
        git status --porcelain | head -10
        echo -e "Branch: ${GREEN}$(git branch --show-current)${NC}"
        echo -e "Last commit: ${GREEN}$(git log -1 --pretty=format:'%h - %s (%cr)')${NC}"
        echo
    fi

    # Rust version
    if command -v rustc >/dev/null 2>&1; then
        echo -e "${YELLOW}Rust Version:${NC} ${GREEN}$(rustc --version)${NC}"
        echo -e "${YELLOW}Cargo Version:${NC} ${GREEN}$(cargo --version)${NC}"
        echo
    fi

    # Project structure
    echo -e "${YELLOW}Project Structure:${NC}"
    echo -e "  📁 Source code: ${GREEN}$(find "$PROJECT_ROOT/src" -name "*.rs" 2>/dev/null | wc -l)${NC} Rust files"
    echo -e "  📁 Examples: ${GREEN}$(find "$PROJECT_ROOT/examples" -name "*.nag" 2>/dev/null | wc -l)${NC} Nagari files"
    echo -e "  📁 Tests: ${GREEN}$(find "$PROJECT_ROOT/tests" -name "*.rs" 2>/dev/null | wc -l)${NC} test files"
    echo -e "  📁 Documentation: ${GREEN}$(find "$PROJECT_ROOT/docs" -name "*.md" 2>/dev/null | wc -l)${NC} markdown files"
    echo

    # Build status
    if [[ -f "$PROJECT_ROOT/target/debug/nagari" ]] || [[ -f "$PROJECT_ROOT/target/debug/nagari.exe" ]]; then
        echo -e "${YELLOW}Build Status:${NC} ${GREEN}Debug build available${NC}"
    else
        echo -e "${YELLOW}Build Status:${NC} ${RED}No debug build found${NC}"
    fi

    if [[ -f "$PROJECT_ROOT/target/release/nagari" ]] || [[ -f "$PROJECT_ROOT/target/release/nagari.exe" ]]; then
        echo -e "${YELLOW}Release Build:${NC} ${GREEN}Available${NC}"
    else
        echo -e "${YELLOW}Release Build:${NC} ${RED}Not built${NC}"
    fi
    echo
}

# Quick development workflow
dev_workflow() {
    echo -e "${CYAN}=== Starting Development Workflow ===${NC}"
    run_tool "setup-dev-env" "$@"
    if [[ $? -eq 0 ]]; then
        run_tool "dev-server" "$@"
    fi
}

# Check workflow (lint + test)
check_workflow() {
    echo -e "${CYAN}=== Running Check Workflow ===${NC}"
    run_tool "lint-check" "$@"
    lint_exit=$?

    run_tool "test-runner" "$@"
    test_exit=$?

    if [[ $lint_exit -eq 0 && $test_exit -eq 0 ]]; then
        echo -e "${GREEN}✅ All checks passed!${NC}"
        return 0
    else
        echo -e "${RED}❌ Some checks failed${NC}"
        return 1
    fi
}

# Ship workflow (lint + test + release)
ship_workflow() {
    echo -e "${CYAN}=== Running Ship Workflow ===${NC}"

    check_workflow "$@"
    if [[ $? -eq 0 ]]; then
        run_tool "release-prep" "$@"
        if [[ $? -eq 0 ]]; then
            echo -e "${GREEN}🚀 Ready to ship!${NC}"
        else
            echo -e "${RED}❌ Release preparation failed${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌ Cannot ship - checks failed${NC}"
        return 1
    fi
}

# Build project
build_project() {
    local build_type="${1:-debug}"
    echo -e "${BLUE}Building project ($build_type)...${NC}"

    cd "$PROJECT_ROOT"

    case "$build_type" in
        "debug"|"dev")
            cargo build
            ;;
        "release"|"prod")
            cargo build --release
            ;;
        *)
            echo -e "${RED}Invalid build type: $build_type${NC}"
            echo -e "Valid types: debug, release"
            return 1
            ;;
    esac
}

# Clean build artifacts
clean_project() {
    echo -e "${BLUE}Cleaning build artifacts...${NC}"
    cd "$PROJECT_ROOT"
    cargo clean

    # Clean additional directories
    [[ -d "dist" ]] && rm -rf dist/
    [[ -d "test-results" ]] && rm -rf test-results/
    [[ -d "lint-results" ]] && rm -rf lint-results/
    [[ -d "coverage" ]] && rm -rf coverage/

    echo -e "${GREEN}✅ Cleaned successfully${NC}"
}

# Watch for changes
watch_project() {
    echo -e "${BLUE}Watching for changes...${NC}"
    cd "$PROJECT_ROOT"

    if command -v cargo-watch >/dev/null 2>&1; then
        cargo watch -x check -x test -x 'run --example hello'
    else
        echo -e "${YELLOW}cargo-watch not installed. Install with: cargo install cargo-watch${NC}"
        return 1
    fi
}

# Main execution
main() {
    if [[ $# -eq 0 ]]; then
        show_banner
        show_usage
        return 0
    fi

    local command="$1"
    shift

    case "$command" in
        "setup")
            run_tool "setup-dev-env" "$@"
            ;;
        "server")
            run_tool "dev-server" "$@"
            ;;
        "test")
            run_tool "test-runner" "$@"
            ;;
        "lint")
            run_tool "lint-check" "$@"
            ;;
        "version")
            run_tool "version-bump" "$@"
            ;;
        "release")
            run_tool "release-prep" "$@"
            ;;
        "build")
            build_project "$@"
            ;;
        "clean")
            clean_project "$@"
            ;;
        "watch")
            watch_project "$@"
            ;;
        "status")
            show_status
            ;;
        "dev")
            dev_workflow "$@"
            ;;
        "check")
            check_workflow "$@"
            ;;
        "ship")
            ship_workflow "$@"
            ;;
        "help"|"-h"|"--help")
            show_banner
            show_usage
            ;;
        *)
            echo -e "${RED}Unknown command: $command${NC}"
            echo -e "Run '$0 help' for usage information"
            return 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"

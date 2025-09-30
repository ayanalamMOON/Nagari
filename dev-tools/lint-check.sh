#!/bin/bash

# Nagari Lint and Format Tool
# Comprehensive code quality checking and automatic formatting

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
LINT_OUTPUT_DIR="$PROJECT_ROOT/lint-results"

# Options
AUTO_FIX=false
CHECK_ONLY=false
VERBOSE=false
STRICT_MODE=false

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  Nagari Lint & Format Tool${NC}"
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

setup_lint_environment() {
    print_info "Setting up lint environment..."

    # Create output directory
    mkdir -p "$LINT_OUTPUT_DIR"
    rm -rf "$LINT_OUTPUT_DIR"/*

    cd "$PROJECT_ROOT"

    print_success "Lint environment ready"
}

check_rust_formatting() {
    print_section "Rust Code Formatting"

    if $CHECK_ONLY; then
        print_info "Checking Rust code formatting..."

        if cargo fmt --check 2>&1 | tee "$LINT_OUTPUT_DIR/rust_fmt_check.log"; then
            print_success "Rust code formatting is correct"
            return 0
        else
            print_error "Rust code formatting issues found"
            print_info "Run with --fix to automatically format code"
            return 1
        fi
    else
        print_info "Formatting Rust code..."

        if cargo fmt 2>&1 | tee "$LINT_OUTPUT_DIR/rust_fmt.log"; then
            print_success "Rust code formatted successfully"
            return 0
        else
            print_error "Rust formatting failed"
            return 1
        fi
    fi
}

run_rust_linting() {
    print_section "Rust Code Linting"
    print_info "Running Clippy linter..."

    local clippy_args=("--all-targets" "--all-features")

    if $STRICT_MODE; then
        clippy_args+=("--" "-D" "warnings" "-D" "clippy::pedantic")
    else
        clippy_args+=("--" "-D" "warnings")
    fi

    if $VERBOSE; then
        clippy_args+=("-v")
    fi

    if cargo clippy "${clippy_args[@]}" 2>&1 | tee "$LINT_OUTPUT_DIR/clippy.log"; then
        print_success "Rust linting passed"

        # Extract suggestions count
        local suggestions=$(grep -c "warning:" "$LINT_OUTPUT_DIR/clippy.log" || echo "0")
        echo "Clippy suggestions: $suggestions" > "$LINT_OUTPUT_DIR/clippy_summary.txt"

        return 0
    else
        print_error "Rust linting failed"

        # Extract error count
        local errors=$(grep -c "error:" "$LINT_OUTPUT_DIR/clippy.log" || echo "0")
        local warnings=$(grep -c "warning:" "$LINT_OUTPUT_DIR/clippy.log" || echo "0")
        echo "Clippy errors: $errors" > "$LINT_OUTPUT_DIR/clippy_summary.txt"
        echo "Clippy warnings: $warnings" >> "$LINT_OUTPUT_DIR/clippy_summary.txt"

        return 1
    fi
}

check_nagari_files() {
    print_section "Nagari Source Files"
    print_info "Checking .nag files..."

    local nag_files_count=0
    local valid_files=0
    local invalid_files=()

    # Check examples
    if [ -d "$PROJECT_ROOT/examples" ]; then
        for file in "$PROJECT_ROOT/examples"/*.nag; do
            if [ -f "$file" ]; then
                local filename=$(basename "$file")
                nag_files_count=$((nag_files_count + 1))

                print_info "Checking $filename..."

                # Basic syntax check using the compiler
                if timeout 30s cargo run --quiet -- check "$file" 2>&1 | tee "$LINT_OUTPUT_DIR/nag_${filename}.log"; then
                    print_success "✓ $filename"
                    valid_files=$((valid_files + 1))
                else
                    print_error "✗ $filename"
                    invalid_files+=("$filename")
                fi
            fi
        done
    fi

    # Check samples
    if [ -d "$PROJECT_ROOT/samples" ]; then
        for file in "$PROJECT_ROOT/samples"/*.nag; do
            if [ -f "$file" ]; then
                local filename=$(basename "$file")
                nag_files_count=$((nag_files_count + 1))

                print_info "Checking $filename..."

                if timeout 30s cargo run --quiet -- check "$file" 2>&1 | tee "$LINT_OUTPUT_DIR/nag_${filename}.log"; then
                    print_success "✓ $filename"
                    valid_files=$((valid_files + 1))
                else
                    print_error "✗ $filename"
                    invalid_files+=("$filename")
                fi
            fi
        done
    fi

    # Check test files
    if [ -d "$PROJECT_ROOT/tests" ]; then
        for file in "$PROJECT_ROOT/tests"/*.nag; do
            if [ -f "$file" ]; then
                local filename=$(basename "$file")
                nag_files_count=$((nag_files_count + 1))

                print_info "Checking $filename..."

                if timeout 30s cargo run --quiet -- check "$file" 2>&1 | tee "$LINT_OUTPUT_DIR/nag_${filename}.log"; then
                    print_success "✓ $filename"
                    valid_files=$((valid_files + 1))
                else
                    print_error "✗ $filename"
                    invalid_files+=("$filename")
                fi
            fi
        done
    fi

    # Summary
    echo "Total .nag files: $nag_files_count" > "$LINT_OUTPUT_DIR/nagari_files_summary.txt"
    echo "Valid files: $valid_files" >> "$LINT_OUTPUT_DIR/nagari_files_summary.txt"
    echo "Invalid files: $((nag_files_count - valid_files))" >> "$LINT_OUTPUT_DIR/nagari_files_summary.txt"

    if [ ${#invalid_files[@]} -eq 0 ]; then
        print_success "All .nag files are valid ($valid_files/$nag_files_count)"
        return 0
    else
        print_error "Some .nag files have issues: ${invalid_files[*]}"
        return 1
    fi
}

check_documentation() {
    print_section "Documentation"
    print_info "Checking documentation files..."

    local issues_found=0

    # Check README files
    for readme in "$PROJECT_ROOT"/README.md "$PROJECT_ROOT"/*/README.md; do
        if [ -f "$readme" ]; then
            local filename=$(basename "$(dirname "$readme")")/$(basename "$readme")
            print_info "Checking $filename..."

            # Check for basic content
            if [ ! -s "$readme" ]; then
                print_warning "Empty README: $filename"
                issues_found=$((issues_found + 1))
            fi

            # Check for broken links (basic check)
            if grep -q "](.*\.md)" "$readme"; then
                while IFS= read -r line; do
                    local link=$(echo "$line" | grep -o "](.*\.md)" | sed 's/](\(.*\))/\1/')
                    if [ ! -f "$(dirname "$readme")/$link" ] && [ ! -f "$PROJECT_ROOT/$link" ]; then
                        print_warning "Broken link in $filename: $link"
                        issues_found=$((issues_found + 1))
                    fi
                done < <(grep "](.*\.md)" "$readme")
            fi
        fi
    done

    echo "Documentation issues: $issues_found" > "$LINT_OUTPUT_DIR/documentation_summary.txt"

    if [ $issues_found -eq 0 ]; then
        print_success "Documentation checks passed"
        return 0
    else
        print_warning "Found $issues_found documentation issues"
        return 1
    fi
}

check_project_structure() {
    print_section "Project Structure"
    print_info "Checking project structure..."

    local issues_found=0

    # Check required files
    local required_files=("README.md" "LICENSE" "Cargo.toml" ".gitignore")
    for file in "${required_files[@]}"; do
        if [ ! -f "$PROJECT_ROOT/$file" ]; then
            print_warning "Missing required file: $file"
            issues_found=$((issues_found + 1))
        fi
    done

    # Check required directories
    local required_dirs=("src" "docs" "examples")
    for dir in "${required_dirs[@]}"; do
        if [ ! -d "$PROJECT_ROOT/$dir" ]; then
            print_warning "Missing required directory: $dir"
            issues_found=$((issues_found + 1))
        fi
    done

    # Check for build artifacts in git
    if [ -d "$PROJECT_ROOT/.git" ]; then
        if git ls-files | grep -E "\.(log|tmp|cache)$" > /dev/null; then
            print_warning "Build artifacts found in git"
            issues_found=$((issues_found + 1))
        fi
    fi

    # Check for large files
    find "$PROJECT_ROOT" -type f -size +10M 2>/dev/null | while read -r file; do
        if [[ "$file" != *"target/"* ]] && [[ "$file" != *".git/"* ]]; then
            print_warning "Large file found: $(basename "$file") ($(du -h "$file" | cut -f1))"
            issues_found=$((issues_found + 1))
        fi
    done

    echo "Structure issues: $issues_found" > "$LINT_OUTPUT_DIR/structure_summary.txt"

    if [ $issues_found -eq 0 ]; then
        print_success "Project structure is correct"
        return 0
    else
        print_warning "Found $issues_found project structure issues"
        return 1
    fi
}

check_dependencies() {
    print_section "Dependencies"
    print_info "Checking dependencies..."

    # Check for outdated Cargo dependencies
    if command -v cargo-outdated &> /dev/null; then
        print_info "Checking for outdated Rust dependencies..."
        cargo outdated 2>&1 | tee "$LINT_OUTPUT_DIR/cargo_outdated.log"
    else
        print_warning "cargo-outdated not installed. Run: cargo install cargo-outdated"
    fi

    # Check for security vulnerabilities
    if command -v cargo-audit &> /dev/null; then
        print_info "Checking for security vulnerabilities..."
        if cargo audit 2>&1 | tee "$LINT_OUTPUT_DIR/cargo_audit.log"; then
            print_success "No security vulnerabilities found"
        else
            print_warning "Security vulnerabilities detected. Check audit report."
        fi
    else
        print_warning "cargo-audit not installed. Run: cargo install cargo-audit"
    fi

    # Check Node.js dependencies if present
    if [ -f "$PROJECT_ROOT/package.json" ]; then
        print_info "Checking Node.js dependencies..."
        if command -v npm &> /dev/null; then
            npm audit 2>&1 | tee "$LINT_OUTPUT_DIR/npm_audit.log" || true
        fi
    fi

    print_success "Dependency checks completed"
    return 0
}

generate_lint_report() {
    print_section "Lint Report"
    print_info "Generating comprehensive lint report..."

    local report_file="$LINT_OUTPUT_DIR/lint_report.html"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    cat > "$report_file" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Nagari Lint Report</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f8f9fa;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 8px;
            padding: 30px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        .header {
            text-align: center;
            margin-bottom: 30px;
        }
        .header h1 {
            color: #2c3e50;
            margin-bottom: 5px;
        }
        .timestamp {
            color: #6c757d;
            font-size: 14px;
        }
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        .card {
            border: 1px solid #dee2e6;
            border-radius: 8px;
            padding: 20px;
        }
        .card h3 {
            margin-top: 0;
            color: #2c3e50;
        }
        .status {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 12px;
            font-weight: bold;
            text-transform: uppercase;
        }
        .status.passed {
            background: #d1ecf1;
            color: #0c5460;
        }
        .status.warning {
            background: #fff3cd;
            color: #856404;
        }
        .status.failed {
            background: #f8d7da;
            color: #721c24;
        }
        .metric {
            display: flex;
            justify-content: space-between;
            margin: 10px 0;
        }
        .logs {
            margin-top: 30px;
        }
        .log-section {
            margin-bottom: 20px;
        }
        .log-content {
            background: #f8f9fa;
            border: 1px solid #e9ecef;
            border-radius: 4px;
            padding: 15px;
            font-family: monospace;
            font-size: 12px;
            max-height: 200px;
            overflow-y: auto;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔍 Nagari Lint Report</h1>
            <div class="timestamp">Generated on $timestamp</div>
        </div>

        <div class="grid">
EOF

    # Add summary cards for each check
    local checks=("clippy" "nagari_files" "documentation" "structure")
    local overall_status="passed"

    for check in "${checks[@]}"; do
        local summary_file="$LINT_OUTPUT_DIR/${check}_summary.txt"
        if [ -f "$summary_file" ]; then
            local content=$(cat "$summary_file")
            local status="passed"

            # Determine status based on content
            if echo "$content" | grep -q "issues: [1-9]"; then
                status="warning"
                overall_status="warning"
            fi

            if echo "$content" | grep -q "errors: [1-9]"; then
                status="failed"
                overall_status="failed"
            fi

            cat >> "$report_file" << EOF
            <div class="card">
                <h3>${check^} Check</h3>
                <div class="status $status">$status</div>
                <div class="metrics">
EOF

            # Add metrics from summary file
            while IFS= read -r line; do
                if [[ "$line" == *":"* ]]; then
                    local key=$(echo "$line" | cut -d':' -f1)
                    local value=$(echo "$line" | cut -d':' -f2- | sed 's/^ *//')
                    cat >> "$report_file" << EOF
                    <div class="metric">
                        <span>$key:</span>
                        <strong>$value</strong>
                    </div>
EOF
                fi
            done < "$summary_file"

            cat >> "$report_file" << EOF
                </div>
            </div>
EOF
        fi
    done

    cat >> "$report_file" << EOF
        </div>

        <div class="logs">
            <h2>Detailed Logs</h2>
EOF

    # Add detailed logs
    for log_file in "$LINT_OUTPUT_DIR"/*.log; do
        if [ -f "$log_file" ]; then
            local log_type=$(basename "$log_file" ".log")
            cat >> "$report_file" << EOF
            <div class="log-section">
                <h3>${log_type^} Log</h3>
                <div class="log-content">
                    <pre>$(cat "$log_file" | tail -100)</pre>
                </div>
            </div>
EOF
        fi
    done

    cat >> "$report_file" << EOF
        </div>
    </div>
</body>
</html>
EOF

    print_success "Lint report generated: $report_file"

    # Print overall status
    echo
    echo -e "${GREEN}==== LINT SUMMARY ====${NC}"
    case $overall_status in
        "passed")
            print_success "All checks passed! ✨"
            ;;
        "warning")
            print_warning "Some warnings found ⚠️"
            ;;
        "failed")
            print_error "Issues found that need fixing ❌"
            ;;
    esac
    echo -e "Report: ${BLUE}$report_file${NC}"

    return 0
}

main() {
    print_header

    setup_lint_environment

    local exit_code=0

    # Run all checks
    check_rust_formatting || exit_code=1
    run_rust_linting || exit_code=1
    check_nagari_files || exit_code=1
    check_documentation || exit_code=1
    check_project_structure || exit_code=1
    check_dependencies || exit_code=1

    # Generate report
    generate_lint_report

    return $exit_code
}

# Help text
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Nagari Lint & Format Tool"
    echo
    echo "Comprehensive code quality checking and automatic formatting."
    echo
    echo "Usage: $0 [options]"
    echo
    echo "Options:"
    echo "  --check            Check-only mode (no auto-fixing)"
    echo "  --fix              Auto-fix issues where possible"
    echo "  --strict           Enable strict/pedantic linting"
    echo "  --verbose          Enable verbose output"
    echo "  --help, -h         Show this help message"
    echo
    echo "Features:"
    echo "  • Rust code formatting (cargo fmt)"
    echo "  • Rust linting (clippy)"
    echo "  • Nagari source file validation"
    echo "  • Documentation checks"
    echo "  • Project structure validation"
    echo "  • Dependency security audit"
    echo "  • Comprehensive HTML report generation"
    echo
    echo "Examples:"
    echo "  $0                 Run all checks with auto-fix"
    echo "  $0 --check        Check without making changes"
    echo "  $0 --strict       Run with strict linting rules"
    echo
    echo "Results are saved in: $LINT_OUTPUT_DIR"
    exit 0
fi

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --check)
            CHECK_ONLY=true
            shift
            ;;
        --fix)
            AUTO_FIX=true
            CHECK_ONLY=false
            shift
            ;;
        --strict)
            STRICT_MODE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
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

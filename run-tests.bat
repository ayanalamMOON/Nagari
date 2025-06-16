@echo off
REM Comprehensive test runner for Nagari ecosystem (Windows)
setlocal enabledelayedexpansion

echo 🧪 Running Nagari Ecosystem Tests
echo ==================================

REM Function to print status
set "SUCCESS=0"
set "FAILURE=1"

REM Get the project root directory
set "PROJECT_ROOT=%~dp0"
cd /d "%PROJECT_ROOT%"

echo Project root: %PROJECT_ROOT%

REM Test CLI Tool
echo.
echo Testing CLI Tool...
if exist "cli\Cargo.toml" (
    cd cli

    echo Running unit tests...
    cargo test --lib
    if !errorlevel! equ 0 (
        echo ✓ Unit tests for CLI
    ) else (
        echo ✗ Unit tests for CLI
    )

    echo Running integration tests...
    if exist "tests" (
        cargo test --test *
        if !errorlevel! equ 0 (
            echo ✓ Integration tests for CLI
        ) else (
            echo ✗ Integration tests for CLI
        )
    )

    echo Running doc tests...
    cargo test --doc
    if !errorlevel! equ 0 (
        echo ✓ Doc tests for CLI
    ) else (
        echo ✗ Doc tests for CLI
    )

    echo Checking code formatting...
    cargo fmt -- --check
    if !errorlevel! equ 0 (
        echo ✓ Code formatting for CLI
    ) else (
        echo ✗ Code formatting for CLI
    )

    echo Running clippy...
    cargo clippy -- -D warnings
    if !errorlevel! equ 0 (
        echo ✓ Clippy checks for CLI
    ) else (
        echo ✗ Clippy checks for CLI
    )

    cd ..
) else (
    echo No CLI directory found, skipping...
)

REM Test Registry Server
echo.
echo Testing Registry Server...
if exist "registry-server\Cargo.toml" (
    cd registry-server

    echo Running unit tests...
    cargo test --lib
    if !errorlevel! equ 0 (
        echo ✓ Unit tests for Registry Server
    ) else (
        echo ✗ Unit tests for Registry Server
    )

    echo Building registry server...
    cargo build --release
    if !errorlevel! equ 0 (
        echo ✓ Registry server build
    ) else (
        echo ✗ Registry server build
    )

    cd ..
) else (
    echo No registry-server directory found, skipping...
)

REM Test LSP Server
echo.
echo Testing LSP Server...
if exist "lsp-server\Cargo.toml" (
    cd lsp-server

    echo Running unit tests...
    cargo test --lib
    if !errorlevel! equ 0 (
        echo ✓ Unit tests for LSP Server
    ) else (
        echo ✗ Unit tests for LSP Server
    )

    echo Building LSP server...
    cargo build --release
    if !errorlevel! equ 0 (
        echo ✓ LSP server build
    ) else (
        echo ✗ LSP server build
    )

    cd ..
) else (
    echo No lsp-server directory found, skipping...
)

REM Test CLI functionality
echo.
echo Testing CLI functionality...
if exist "cli\target\release\nag.exe" (
    echo Testing CLI help command...
    cli\target\release\nag.exe --help >nul 2>&1
    if !errorlevel! equ 0 (
        echo ✓ CLI help command
    ) else (
        echo ✗ CLI help command
    )
) else (
    echo CLI binary not found, build first
)

echo.
echo 🎉 All tests completed!
echo.
echo Summary:
echo - CLI Tool: Unit, integration, and build tests
echo - Registry Server: Unit tests and build
echo - LSP Server: Unit tests and build
echo - Code formatting and linting checks
echo.
echo Next steps:
echo 1. Review any failed tests above
echo 2. Run manual integration tests
echo 3. Test in different environments

pause

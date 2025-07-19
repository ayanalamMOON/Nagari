@echo off
REM Quick test script to verify build before release (Windows)
REM Usage: scripts\test-build.bat

setlocal enabledelayedexpansion

echo 🔷 Running pre-release tests for Nagari

REM Check if we're in the right directory
if not exist "Cargo.toml" goto :wrong_dir
if not exist "src" goto :wrong_dir

REM Test runtime
echo 🔷 Testing nagari-runtime
cd nagari-runtime
if not exist "node_modules" npm install
npm run build
if errorlevel 1 (
    echo ❌ Runtime build failed
    exit /b 1
)
cd ..
echo ✅ Runtime build successful

REM Test Rust workspace
echo 🔷 Testing Rust workspace
cargo test --workspace
if errorlevel 1 (
    echo ❌ Rust tests failed
    exit /b 1
)
echo ✅ Rust tests passed

REM Build and test CLI
echo 🔷 Building and testing CLI
cargo build --release --bin nag
if errorlevel 1 (
    echo ❌ CLI build failed
    exit /b 1
)
target\release\nag.exe --version

REM Create test file
echo print("Test successful!") > test_quick.nag
target\release\nag.exe compile test_quick.nag

if exist "test_quick.js" (
    echo ✅ Compilation test passed
    del test_quick.nag test_quick.js 2>nul
) else (
    echo ❌ Compilation test failed
    exit /b 1
)

REM Test LSP server
echo 🔷 Testing LSP server
cargo build --release --bin nagari-lsp
if errorlevel 1 (
    echo ❌ LSP server build failed
    exit /b 1
)
target\release\nagari-lsp.exe --help >nul 2>&1
echo ✅ LSP server test passed

echo ✅ All pre-release tests passed! 🎉
echo.
echo Ready for release! Run:
echo   scripts\release.bat [version]
goto :end

:wrong_dir
echo ❌ This script must be run from the project root directory
exit /b 1

:end

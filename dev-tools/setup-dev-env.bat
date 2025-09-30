@echo off
REM Nagari Development Environment Setup (Windows)
REM This script sets up a complete development environment for Nagari

setlocal enabledelayedexpansion

REM Configuration
set "PROJECT_ROOT=%~dp0.."
set "RUST_VERSION=1.70.0"
set "NODE_VERSION=18.0.0"

echo ================================
echo   Nagari Development Setup
echo ================================
echo.

REM Check prerequisites
echo [STEP] Checking prerequisites...

where git >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Git is not installed. Please install Git and try again.
    echo         Download from: https://git-scm.com/
    pause
    exit /b 1
)

where curl >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Curl is not installed. Please install curl and try again.
    pause
    exit /b 1
)

echo [SUCCESS] Prerequisites check passed

REM Setup Rust
echo [STEP] Setting up Rust development environment...

where rustc >nul 2>&1
if %errorlevel% neq 0 (
    echo [INFO] Installing Rust via rustup...
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    call "%USERPROFILE%\.cargo\env.bat"
) else (
    for /f "tokens=2" %%i in ('rustc --version') do set "current_version=%%i"
    echo [INFO] Rust already installed: !current_version!
)

echo [INFO] Updating Rust components...
rustup update
rustup component add clippy rustfmt rust-analyzer

echo [INFO] Installing cargo tools...
cargo install --locked cargo-watch cargo-edit cargo-audit cargo-outdated 2>nul || echo [WARNING] Some cargo tools may have failed to install

echo [SUCCESS] Rust environment ready

REM Setup Node.js
echo [STEP] Setting up Node.js environment...

where node >nul 2>&1
if %errorlevel% neq 0 (
    echo [INFO] Please install Node.js %NODE_VERSION% or later
    echo [INFO] Visit: https://nodejs.org/
    echo [INFO] You can also install with: winget install OpenJS.NodeJS
    pause
) else (
    for /f "tokens=*" %%i in ('node --version') do set "current_version=%%i"
    echo [INFO] Node.js already installed: !current_version!
)

where npm >nul 2>&1
if %errorlevel% equ 0 (
    echo [INFO] Updating npm...
    npm install -g npm@latest 2>nul || echo [WARNING] npm update may have failed
)

echo [SUCCESS] Node.js environment ready

REM Setup project
echo [STEP] Setting up project dependencies...

cd /d "%PROJECT_ROOT%"

echo [INFO] Building Rust project...
cargo check

if exist "nagari-runtime" (
    echo [INFO] Installing runtime dependencies...
    cd nagari-runtime
    npm install
    npm run build
    cd /d "%PROJECT_ROOT%"
)

if exist "vscode-extension" (
    echo [INFO] Installing VS Code extension dependencies...
    cd vscode-extension
    npm install
    cd /d "%PROJECT_ROOT%"
)

echo [SUCCESS] Project dependencies installed

REM Create development configuration
echo [STEP] Creating development configuration...

(
echo {
echo   "environment": "development",
echo   "compiler": {
echo     "debug": true,
echo     "optimize": false,
echo     "emit_source_maps": true
echo   },
echo   "runtime": {
echo     "enable_debugging": true,
echo     "verbose_logging": true
echo   },
echo   "testing": {
echo     "parallel": true,
echo     "coverage": true,
echo     "watch_mode": true
echo   },
echo   "lsp": {
echo     "enable_diagnostics": true,
echo     "completion": true,
echo     "hover_info": true
echo   },
echo   "tools": {
echo     "auto_format": true,
echo     "lint_on_save": true,
echo     "pre_commit_hooks": true
echo   }
echo }
) > ".nagari-dev.json"

echo [SUCCESS] Development configuration created

REM Setup Git hooks
echo [STEP] Setting up Git hooks...

if not exist ".git\hooks" mkdir .git\hooks

(
echo @echo off
echo echo Running pre-commit checks...
echo.
echo REM Format code
echo cargo fmt --check
echo if %%errorlevel%% neq 0 ^(
echo     echo Code formatting check failed. Run 'cargo fmt' to fix.
echo     exit /b 1
echo ^)
echo.
echo REM Lint code
echo cargo clippy -- -D warnings
echo if %%errorlevel%% neq 0 ^(
echo     echo Linting failed. Fix clippy warnings before committing.
echo     exit /b 1
echo ^)
echo.
echo REM Run tests
echo cargo test
echo if %%errorlevel%% neq 0 ^(
echo     echo Tests failed. Fix failing tests before committing.
echo     exit /b 1
echo ^)
echo.
echo echo Pre-commit checks passed!
) > ".git\hooks\pre-commit.bat"

echo [SUCCESS] Git hooks installed

REM Setup IDE
echo [STEP] Setting up IDE configuration...

if not exist ".vscode" mkdir .vscode

(
echo {
echo     "rust-analyzer.checkOnSave.command": "clippy",
echo     "rust-analyzer.cargo.features": "all",
echo     "rust-analyzer.completion.addCallParentheses": false,
echo     "editor.formatOnSave": true,
echo     "editor.codeActionsOnSave": {
echo         "source.fixAll": true
echo     },
echo     "files.associations": {
echo         "*.nag": "python"
echo     },
echo     "typescript.preferences.includePackageJsonAutoImports": "on",
echo     "npm.enableScriptExplorer": true,
echo     "terminal.integrated.defaultProfile.windows": "Git Bash"
echo }
) > ".vscode\settings.json"

(
echo {
echo     "recommendations": [
echo         "rust-lang.rust-analyzer",
echo         "ms-vscode.vscode-typescript-next",
echo         "ms-python.python",
echo         "bradlc.vscode-tailwindcss",
echo         "GitHub.copilot",
echo         "ms-vscode.test-adapter-converter",
echo         "hbenl.vscode-test-explorer"
echo     ]
echo }
) > ".vscode\extensions.json"

echo [SUCCESS] IDE configuration created

REM Create development scripts
echo [STEP] Creating development scripts...

(
echo @echo off
echo REM Quick development commands
echo.
echo if "%%1"=="build" ^(
echo     cargo build
echo     goto :eof
echo ^)
echo if "%%1"=="test" ^(
echo     cargo test
echo     goto :eof
echo ^)
echo if "%%1"=="run" ^(
echo     shift
echo     cargo run -- %%*
echo     goto :eof
echo ^)
echo if "%%1"=="format" ^(
echo     cargo fmt
echo     goto :eof
echo ^)
echo if "%%1"=="lint" ^(
echo     cargo clippy
echo     goto :eof
echo ^)
echo if "%%1"=="clean" ^(
echo     cargo clean
echo     if exist dist rmdir /s /q dist
echo     goto :eof
echo ^)
echo if "%%1"=="watch" ^(
echo     cargo watch -x check -x test
echo     goto :eof
echo ^)
echo.
echo echo Usage: %%0 {build^|test^|run^|format^|lint^|clean^|watch}
echo echo.
echo echo Commands:
echo echo   build    - Build the project
echo echo   test     - Run tests
echo echo   run      - Run the CLI ^(pass arguments after 'run'^)
echo echo   format   - Format code
echo echo   lint     - Run linter
echo echo   clean    - Clean build artifacts
echo echo   watch    - Watch for changes and run checks
) > "dev.bat"

echo [SUCCESS] Development scripts created

REM Print summary
echo.
echo ================================
echo   Setup Complete!
echo ================================
echo.
echo Next steps:
echo 1. Run tests: dev.bat test
echo 2. Start development: dev.bat watch
echo 3. Build project: dev.bat build
echo 4. Open in VS Code and install recommended extensions
echo.
echo Available tools:
echo • dev.bat - Quick development commands
echo • dev-tools\ - Complete development toolkit
echo • .nagari-dev.json - Development configuration
echo.
echo Happy coding! 🚀

pause

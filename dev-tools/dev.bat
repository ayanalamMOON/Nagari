@echo off
setlocal enabledelayedexpansion

REM Nagari Development Master Launcher - Windows Edition
REM Entry point for all development tasks

set "SCRIPT_DIR=%~dp0"
set "PROJECT_ROOT=%SCRIPT_DIR%..\"
set "CONFIG_FILE=%SCRIPT_DIR%config.json"

REM Main execution function
if "%1"=="" (
    call :show_banner
    call :show_usage
    goto :eof
)

set "COMMAND=%1"
shift

if "%COMMAND%"=="setup" (
    call :run_tool "setup-dev-env" %*
) else if "%COMMAND%"=="server" (
    call :run_tool "dev-server" %*
) else if "%COMMAND%"=="test" (
    call :run_tool "test-runner" %*
) else if "%COMMAND%"=="lint" (
    call :run_tool "lint-check" %*
) else if "%COMMAND%"=="version" (
    call :run_tool "version-bump" %*
) else if "%COMMAND%"=="release" (
    call :run_tool "release-prep" %*
) else if "%COMMAND%"=="build" (
    call :build_project %*
) else if "%COMMAND%"=="clean" (
    call :clean_project %*
) else if "%COMMAND%"=="watch" (
    call :watch_project %*
) else if "%COMMAND%"=="status" (
    call :show_status
) else if "%COMMAND%"=="dev" (
    call :dev_workflow %*
) else if "%COMMAND%"=="check" (
    call :check_workflow %*
) else if "%COMMAND%"=="ship" (
    call :ship_workflow %*
) else if "%COMMAND%"=="help" (
    call :show_banner
    call :show_usage
) else if "%COMMAND%"=="-h" (
    call :show_banner
    call :show_usage
) else if "%COMMAND%"=="--help" (
    call :show_banner
    call :show_usage
) else (
    echo [91mUnknown command: %COMMAND%[0m
    echo Run '%~nx0 help' for usage information
    exit /b 1
)

goto :eof

REM Show banner
:show_banner
echo [95m
echo ╔═══════════════════════════════════════════════════════════════════════════════╗
echo ║                            🚀 NAGARI DEV TOOLS 🚀                            ║
echo ║                     Comprehensive Development Toolkit                        ║
echo ╚═══════════════════════════════════════════════════════════════════════════════╝
echo [0m
goto :eof

REM Show usage information
:show_usage
echo [96mUsage: %~nx0 [command] [options][0m
echo.
echo [93mAvailable Commands:[0m
echo   [92msetup[0m      - Setup development environment
echo   [92mserver[0m     - Start development server with hot reload
echo   [92mtest[0m       - Run comprehensive test suite
echo   [92mlint[0m       - Run code quality checks and formatting
echo   [92mversion[0m    - Bump version (major^|minor^|patch)
echo   [92mrelease[0m    - Prepare release packages
echo   [92mbuild[0m      - Build project (debug^|release)
echo   [92mclean[0m      - Clean build artifacts
echo   [92mwatch[0m      - Watch for changes and rebuild
echo   [92mstatus[0m     - Show project status
echo   [92mhelp[0m       - Show this help message
echo.
echo [93mQuick Commands:[0m
echo   [94mdev[0m        - Equivalent to: setup ^&^& server
echo   [94mcheck[0m      - Equivalent to: lint ^&^& test
echo   [94mship[0m       - Equivalent to: lint ^&^& test ^&^& release
echo.
echo [93mExamples:[0m
echo   %~nx0 setup                    # Setup development environment
echo   %~nx0 server --port 4000       # Start server on port 4000
echo   %~nx0 test --coverage          # Run tests with coverage
echo   %~nx0 version minor            # Bump minor version
echo   %~nx0 release --all-platforms  # Build for all platforms
echo.
goto :eof

REM Execute a development tool
:run_tool
set "TOOL_NAME=%~1"
set "SCRIPT_PATH=%SCRIPT_DIR%%TOOL_NAME%.bat"

if not exist "%SCRIPT_PATH%" (
    echo [91mError: Tool '%TOOL_NAME%' not found at %SCRIPT_PATH%[0m
    exit /b 1
)

echo [94mRunning: %TOOL_NAME%[0m
call "%SCRIPT_PATH%" %*
goto :eof

REM Show project status
:show_status
echo [96m=== Nagari Project Status ===[0m
echo.

REM Git status
where git >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo [93mGit Status:[0m
    cd /d "%PROJECT_ROOT%"
    git status --porcelain | head -10
    for /f "tokens=*" %%i in ('git branch --show-current') do set "CURRENT_BRANCH=%%i"
    echo Branch: [92m!CURRENT_BRANCH![0m
    for /f "tokens=*" %%i in ('git log -1 --pretty^=format:"%%h - %%s (%%cr)"') do set "LAST_COMMIT=%%i"
    echo Last commit: [92m!LAST_COMMIT![0m
    echo.
)

REM Rust version
where rustc >nul 2>&1
if %ERRORLEVEL% equ 0 (
    for /f "tokens=*" %%i in ('rustc --version') do set "RUST_VERSION=%%i"
    echo [93mRust Version:[0m [92m!RUST_VERSION![0m
    for /f "tokens=*" %%i in ('cargo --version') do set "CARGO_VERSION=%%i"
    echo [93mCargo Version:[0m [92m!CARGO_VERSION![0m
    echo.
)

REM Project structure
echo [93mProject Structure:[0m
if exist "%PROJECT_ROOT%src" (
    for /f %%i in ('dir /s /b "%PROJECT_ROOT%src\*.rs" 2^>nul ^| find /c /v ""') do set "RUST_FILES=%%i"
    echo   📁 Source code: [92m!RUST_FILES![0m Rust files
)
if exist "%PROJECT_ROOT%examples" (
    for /f %%i in ('dir /s /b "%PROJECT_ROOT%examples\*.nag" 2^>nul ^| find /c /v ""') do set "NAG_FILES=%%i"
    echo   📁 Examples: [92m!NAG_FILES![0m Nagari files
)
if exist "%PROJECT_ROOT%tests" (
    for /f %%i in ('dir /s /b "%PROJECT_ROOT%tests\*.rs" 2^>nul ^| find /c /v ""') do set "TEST_FILES=%%i"
    echo   📁 Tests: [92m!TEST_FILES![0m test files
)
if exist "%PROJECT_ROOT%docs" (
    for /f %%i in ('dir /s /b "%PROJECT_ROOT%docs\*.md" 2^>nul ^| find /c /v ""') do set "DOC_FILES=%%i"
    echo   📁 Documentation: [92m!DOC_FILES![0m markdown files
)
echo.

REM Build status
if exist "%PROJECT_ROOT%target\debug\nagari.exe" (
    echo [93mBuild Status:[0m [92mDebug build available[0m
) else (
    echo [93mBuild Status:[0m [91mNo debug build found[0m
)

if exist "%PROJECT_ROOT%target\release\nagari.exe" (
    echo [93mRelease Build:[0m [92mAvailable[0m
) else (
    echo [93mRelease Build:[0m [91mNot built[0m
)
echo.
goto :eof

REM Quick development workflow
:dev_workflow
echo [96m=== Starting Development Workflow ===[0m
call :run_tool "setup-dev-env" %*
if %ERRORLEVEL% equ 0 (
    call :run_tool "dev-server" %*
)
goto :eof

REM Check workflow (lint + test)
:check_workflow
echo [96m=== Running Check Workflow ===[0m
call :run_tool "lint-check" %*
set "LINT_EXIT=%ERRORLEVEL%"

call :run_tool "test-runner" %*
set "TEST_EXIT=%ERRORLEVEL%"

if !LINT_EXIT! equ 0 if !TEST_EXIT! equ 0 (
    echo [92m✅ All checks passed![0m
    exit /b 0
) else (
    echo [91m❌ Some checks failed[0m
    exit /b 1
)
goto :eof

REM Ship workflow (lint + test + release)
:ship_workflow
echo [96m=== Running Ship Workflow ===[0m

call :check_workflow %*
if %ERRORLEVEL% equ 0 (
    call :run_tool "release-prep" %*
    if %ERRORLEVEL% equ 0 (
        echo [92m🚀 Ready to ship![0m
    ) else (
        echo [91m❌ Release preparation failed[0m
        exit /b 1
    )
) else (
    echo [91m❌ Cannot ship - checks failed[0m
    exit /b 1
)
goto :eof

REM Build project
:build_project
set "BUILD_TYPE=%1"
if "%BUILD_TYPE%"=="" set "BUILD_TYPE=debug"

echo [94mBuilding project (%BUILD_TYPE%)...[0m

cd /d "%PROJECT_ROOT%"

if "%BUILD_TYPE%"=="debug" (
    cargo build
) else if "%BUILD_TYPE%"=="dev" (
    cargo build
) else if "%BUILD_TYPE%"=="release" (
    cargo build --release
) else if "%BUILD_TYPE%"=="prod" (
    cargo build --release
) else (
    echo [91mInvalid build type: %BUILD_TYPE%[0m
    echo Valid types: debug, release
    exit /b 1
)
goto :eof

REM Clean build artifacts
:clean_project
echo [94mCleaning build artifacts...[0m
cd /d "%PROJECT_ROOT%"
cargo clean

REM Clean additional directories
if exist "dist" rmdir /s /q "dist"
if exist "test-results" rmdir /s /q "test-results"
if exist "lint-results" rmdir /s /q "lint-results"
if exist "coverage" rmdir /s /q "coverage"

echo [92m✅ Cleaned successfully[0m
goto :eof

REM Watch for changes
:watch_project
echo [94mWatching for changes...[0m
cd /d "%PROJECT_ROOT%"

where cargo-watch >nul 2>&1
if %ERRORLEVEL% equ 0 (
    cargo watch -x check -x test -x "run --example hello"
) else (
    echo [93mcargo-watch not installed. Install with: cargo install cargo-watch[0m
    exit /b 1
)
goto :eof

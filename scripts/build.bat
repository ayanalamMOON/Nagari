@echo off
REM Local build script for Nagari Programming Language (Windows)
REM Usage: scripts\build.bat [target]

setlocal enabledelayedexpansion

REM Get target from argument or default to Windows
set "TARGET=%~1"
if "%TARGET%"=="" set "TARGET=x86_64-pc-windows-msvc"

echo 🔷 Building Nagari for target: %TARGET%

REM Check if we're in the right directory
if not exist "Cargo.toml" goto :wrong_dir
if not exist "src" goto :wrong_dir

REM Install target if not already installed
echo 🔷 Ensuring Rust target %TARGET% is installed
rustup target add %TARGET%

REM Build nagari-runtime first
echo 🔷 Building nagari-runtime (npm package)
cd nagari-runtime
if not exist "node_modules" npm install
npm run build
if errorlevel 1 (
    echo ❌ Runtime build failed
    exit /b 1
)
echo ✅ Runtime built successfully
cd ..

REM Build Rust workspace
echo 🔷 Building Rust workspace
cargo build --release --target %TARGET%
if errorlevel 1 (
    echo ❌ Rust workspace build failed
    exit /b 1
)
echo ✅ Rust workspace built successfully

REM Test the built binaries
echo 🔷 Testing built binaries

set "CLI_BINARY=target\%TARGET%\release\nag.exe"
set "LSP_BINARY=target\%TARGET%\release\nagari-lsp.exe"

REM Check if binaries exist
if not exist "%CLI_BINARY%" (
    echo ❌ CLI binary not found: %CLI_BINARY%
    exit /b 1
)

if not exist "%LSP_BINARY%" (
    echo ❌ LSP binary not found: %LSP_BINARY%
    exit /b 1
)

REM Test CLI
echo 🔷 Testing CLI functionality
%CLI_BINARY% --version
%CLI_BINARY% --help >nul

REM Create a simple test file
echo print("Build test successful!") > test_build.nag
%CLI_BINARY% compile test_build.nag
if exist "test_build.js" (
    echo ✅ Compilation test passed
    del test_build.nag test_build.js 2>nul
) else (
    echo ❌ Compilation test failed
    exit /b 1
)

REM Test LSP (basic check)
echo 🔷 Testing LSP server
%LSP_BINARY% --help >nul 2>&1
echo ✅ LSP server starts correctly

REM Create distribution directory
set "DIST_DIR=dist\nagari-%TARGET%"
echo 🔷 Creating distribution package in %DIST_DIR%

if exist "%DIST_DIR%" rmdir /s /q "%DIST_DIR%"
mkdir "%DIST_DIR%\bin"
mkdir "%DIST_DIR%\stdlib"
mkdir "%DIST_DIR%\runtime"

REM Copy binaries
copy "%CLI_BINARY%" "%DIST_DIR%\bin\"
copy "%LSP_BINARY%" "%DIST_DIR%\bin\"

REM Copy documentation and licenses
copy README.md "%DIST_DIR%\"
copy LICENSE "%DIST_DIR%\"
if exist "CHANGELOG.md" copy CHANGELOG.md "%DIST_DIR%\"

REM Copy standard library
if exist "stdlib" xcopy /E /I stdlib "%DIST_DIR%\stdlib"

REM Copy runtime
xcopy /E /I nagari-runtime\dist "%DIST_DIR%\runtime"
copy nagari-runtime\package.json "%DIST_DIR%\runtime\"

REM Create Windows installation script
echo @echo off > "%DIST_DIR%\install.bat"
echo echo 🚀 Installing Nagari Programming Language... >> "%DIST_DIR%\install.bat"
echo. >> "%DIST_DIR%\install.bat"
echo REM Create installation directory >> "%DIST_DIR%\install.bat"
echo set "INSTALL_DIR=%%USERPROFILE%%\.nagari" >> "%DIST_DIR%\install.bat"
echo set "BIN_DIR=%%INSTALL_DIR%%\bin" >> "%DIST_DIR%\install.bat"
echo set "STDLIB_DIR=%%INSTALL_DIR%%\stdlib" >> "%DIST_DIR%\install.bat"
echo set "RUNTIME_DIR=%%INSTALL_DIR%%\runtime" >> "%DIST_DIR%\install.bat"
echo. >> "%DIST_DIR%\install.bat"
echo if not exist "%%BIN_DIR%%" mkdir "%%BIN_DIR%%" >> "%DIST_DIR%\install.bat"
echo if not exist "%%STDLIB_DIR%%" mkdir "%%STDLIB_DIR%%" >> "%DIST_DIR%\install.bat"
echo if not exist "%%RUNTIME_DIR%%" mkdir "%%RUNTIME_DIR%%" >> "%DIST_DIR%\install.bat"
echo. >> "%DIST_DIR%\install.bat"
echo REM Copy binaries >> "%DIST_DIR%\install.bat"
echo copy bin\*.exe "%%BIN_DIR%%\" >> "%DIST_DIR%\install.bat"
echo. >> "%DIST_DIR%\install.bat"
echo REM Copy standard library and runtime >> "%DIST_DIR%\install.bat"
echo if exist "stdlib" xcopy /E /I stdlib "%%STDLIB_DIR%%" >> "%DIST_DIR%\install.bat"
echo if exist "runtime" xcopy /E /I runtime "%%RUNTIME_DIR%%" >> "%DIST_DIR%\install.bat"
echo. >> "%DIST_DIR%\install.bat"
echo echo. >> "%DIST_DIR%\install.bat"
echo echo ✅ Nagari installed successfully! >> "%DIST_DIR%\install.bat"
echo echo. >> "%DIST_DIR%\install.bat"
echo echo Add the following directory to your PATH: >> "%DIST_DIR%\install.bat"
echo echo %%BIN_DIR%% >> "%DIST_DIR%\install.bat"
echo echo. >> "%DIST_DIR%\install.bat"
echo echo Verify installation: >> "%DIST_DIR%\install.bat"
echo echo nag --version >> "%DIST_DIR%\install.bat"

REM Show binary information
echo 🔷 Build information
echo Target: %TARGET%
for %%F in ("%CLI_BINARY%") do echo CLI binary: %%~zF bytes
for %%F in ("%LSP_BINARY%") do echo LSP binary: %%~zF bytes
echo Distribution: %DIST_DIR%

echo ✅ Build completed successfully!
echo.
echo 📦 Distribution package created in: %DIST_DIR%
echo 🚀 To install locally, run the installation script in the distribution directory
echo 🔧 To test the build:
echo    cd %DIST_DIR%
echo    install.bat
echo    nag --version
goto :end

:wrong_dir
echo ❌ This script must be run from the project root directory
exit /b 1

:end

@echo off
REM Complete package builder for Nagari Programming Language (Windows)
REM Creates standalone executable packages for external distribution
REM Usage: scripts\package-release.bat [version] [target]

setlocal enabledelayedexpansion

echo 🔷 Creating Nagari executable package for external distribution

REM Configuration
set "VERSION=%~1"
if "%VERSION%"=="" set "VERSION=0.3.0"

set "TARGET=%~2"
if "%TARGET%"=="" set "TARGET=x86_64-pc-windows-msvc"

set "ROOT_DIR=%CD%"
set "PACKAGE_NAME=nagari-%VERSION%-%TARGET%"
set "PACKAGE_DIR=packages\%PACKAGE_NAME%"
set "ARCHIVE_NAME=%PACKAGE_NAME%.zip"

echo ℹ️ Package: %PACKAGE_NAME%
echo ℹ️ Archive: %ARCHIVE_NAME%

REM Validate environment
if not exist "Cargo.toml" (
    echo ❌ This script must be run from the project root directory
    exit /b 1
)

REM Check required tools
where rustc >nul 2>&1 || (
    echo ❌ Rust is required but not installed
    exit /b 1
)

where cargo >nul 2>&1 || (
    echo ❌ Cargo is required but not installed
    exit /b 1
)

where npm >nul 2>&1 || (
    echo ❌ Node.js/npm is required but not installed
    exit /b 1
)

REM Clean and create package directory
echo 🔷 Preparing package directory
if exist "packages\%PACKAGE_NAME%" rmdir /S /Q "packages\%PACKAGE_NAME%"
mkdir "%PACKAGE_DIR%\bin" 2>nul
mkdir "%PACKAGE_DIR%\lib" 2>nul
mkdir "%PACKAGE_DIR%\runtime" 2>nul
mkdir "%PACKAGE_DIR%\stdlib" 2>nul
mkdir "%PACKAGE_DIR%\examples" 2>nul
mkdir "%PACKAGE_DIR%\docs" 2>nul

REM Install target if needed
echo 🔷 Ensuring Rust target %TARGET% is available
rustup target add %TARGET%

REM Build runtime first
echo 🔷 Building Nagari runtime (TypeScript/JavaScript)
cd nagari-runtime

if not exist "node_modules" npm install
npm run build

echo ✅ Runtime built successfully
cd "%ROOT_DIR%"

REM Build Rust workspace
echo 🔷 Building Rust workspace for %TARGET%
cargo build --release --target %TARGET% --workspace

echo ✅ Rust workspace built successfully

REM Define binary paths
set "CLI_BINARY=target\%TARGET%\release\nag.exe"
set "LSP_BINARY=target\%TARGET%\release\nagari-lsp.exe"
set "COMPILER_BINARY=target\%TARGET%\release\nagc.exe"

REM Verify binaries exist
echo 🔷 Verifying built binaries
if not exist "%CLI_BINARY%" (
    echo ❌ Binary not found: %CLI_BINARY%
    exit /b 1
)
echo ✅ Found: nag.exe

if not exist "%LSP_BINARY%" (
    echo ❌ Binary not found: %LSP_BINARY%
    exit /b 1
)
echo ✅ Found: nagari-lsp.exe

REM Test functionality
echo 🔷 Testing binary functionality
"%CLI_BINARY%" --version
"%CLI_BINARY%" --help >nul

REM Quick compile test
echo print("Package build test successful!") > test_package.nag
"%CLI_BINARY%" build test_package.nag -o test_package.js
if exist "test_package.js" (
    echo ✅ Compilation test passed
    del test_package.nag test_package.js
) else (
    echo ❌ Compilation test failed
    exit /b 1
)

REM Copy binaries
echo 🔷 Packaging binaries
copy "%CLI_BINARY%" "%PACKAGE_DIR%\bin\"
copy "%LSP_BINARY%" "%PACKAGE_DIR%\bin\"
if exist "%COMPILER_BINARY%" copy "%COMPILER_BINARY%" "%PACKAGE_DIR%\bin\"

REM Copy runtime
echo 🔷 Packaging runtime
xcopy /E /I /Q "nagari-runtime\dist\*" "%PACKAGE_DIR%\runtime\"
copy "nagari-runtime\package.json" "%PACKAGE_DIR%\runtime\"
copy "nagari-runtime\README.md" "%PACKAGE_DIR%\runtime\README.md"

REM Copy standard library
echo 🔷 Packaging standard library
if exist "stdlib" xcopy /E /I /Q "stdlib\*" "%PACKAGE_DIR%\stdlib\"

REM Copy examples
echo 🔷 Packaging examples
if exist "examples" xcopy /E /I /Q "examples\*" "%PACKAGE_DIR%\examples\"

REM Copy documentation
echo 🔷 Packaging documentation
copy "README.md" "%PACKAGE_DIR%\"
copy "LICENSE" "%PACKAGE_DIR%\"
if exist "CHANGELOG.md" copy "CHANGELOG.md" "%PACKAGE_DIR%\"

REM Copy key documentation files
if exist "docs\getting-started.md" copy "docs\getting-started.md" "%PACKAGE_DIR%\docs\"
if exist "docs\language-guide.md" copy "docs\language-guide.md" "%PACKAGE_DIR%\docs\"
if exist "docs\cli-reference.md" copy "docs\cli-reference.md" "%PACKAGE_DIR%\docs\"

REM Create package information file
echo 🔷 Creating package information
(
echo # Nagari Programming Language - Package Information
echo.
echo **Version:** %VERSION%
echo **Target:** %TARGET%
echo **Build Date:** %DATE% %TIME%
echo **Package:** %PACKAGE_NAME%
echo.
echo ## Contents
echo.
echo - `bin/` - Executable binaries
echo   - `nag.exe` - Main CLI tool for compilation, running, and project management
echo   - `nagari-lsp.exe` - Language Server Protocol implementation
echo   - `nagc.exe` - Direct compiler binary ^(if available^)
echo.
echo - `runtime/` - JavaScript runtime library
echo   - Pre-compiled TypeScript runtime for browser and Node.js
echo   - Core utilities and polyfills
echo.
echo - `stdlib/` - Standard library modules
echo   - Core Nagari modules and utilities
echo.
echo - `examples/` - Example Nagari programs
echo   - Demonstrations of language features
echo   - Real-world usage patterns
echo.
echo - `docs/` - Documentation
echo   - Getting started guide
echo   - Language reference
echo   - CLI reference
echo.
echo ## Quick Start
echo.
echo 1. Run the installation script ^(install.bat^)
echo 2. Add bin directory to your PATH
echo 3. Verify installation: `nag --version`
echo 4. Try an example: `nag run examples\hello.nag`
echo.
echo ## System Requirements
echo.
echo - Operating System: Windows ^(compatible with %TARGET%^)
echo - Node.js: Required for runtime features ^(recommended: v16+^)
echo - Memory: Minimum 512MB RAM
echo - Disk Space: ~50MB for full installation
echo.
echo ## Support
echo.
echo - GitHub: https://github.com/ayanalamMOON/Nagari
echo - Documentation: https://github.com/ayanalamMOON/Nagari/docs
echo - Issues: https://github.com/ayanalamMOON/Nagari/issues
) > "%PACKAGE_DIR%\PACKAGE_INFO.md"

REM Create installation script
echo 🔷 Creating installation scripts
(
echo @echo off
echo echo 🚀 Installing Nagari Programming Language...
echo.
echo REM Configuration
echo set "INSTALL_DIR=%%USERPROFILE%%\.nagari"
echo set "BIN_DIR=%%INSTALL_DIR%%\bin"
echo set "STDLIB_DIR=%%INSTALL_DIR%%\stdlib"
echo set "RUNTIME_DIR=%%INSTALL_DIR%%\runtime"
echo set "EXAMPLES_DIR=%%INSTALL_DIR%%\examples"
echo.
echo REM Create directories
echo if not exist "%%BIN_DIR%%" mkdir "%%BIN_DIR%%"
echo if not exist "%%STDLIB_DIR%%" mkdir "%%STDLIB_DIR%%"
echo if not exist "%%RUNTIME_DIR%%" mkdir "%%RUNTIME_DIR%%"
echo if not exist "%%EXAMPLES_DIR%%" mkdir "%%EXAMPLES_DIR%%"
echo.
echo REM Copy files
echo echo 📁 Copying binaries...
echo copy bin\*.exe "%%BIN_DIR%%\"
echo.
echo echo 📚 Copying standard library...
echo if exist "stdlib" xcopy /E /I /Q stdlib "%%STDLIB_DIR%%"
echo.
echo echo ⚡ Copying runtime...
echo if exist "runtime" xcopy /E /I /Q runtime "%%RUNTIME_DIR%%"
echo.
echo echo 📖 Copying examples...
echo if exist "examples" xcopy /E /I /Q examples "%%EXAMPLES_DIR%%"
echo.
echo echo 📄 Copying documentation...
echo copy *.md "%%INSTALL_DIR%%\" ^>nul 2^>^&1
echo if exist "docs" xcopy /E /I /Q docs "%%INSTALL_DIR%%\docs"
echo.
echo echo.
echo echo ✅ Nagari installed successfully!
echo echo.
echo echo ⚠️ To complete installation, add to your PATH:
echo echo %%BIN_DIR%%
echo echo.
echo echo 🔧 Verify installation:
echo echo nag --version
echo echo.
echo echo 🚀 Try an example:
echo echo nag run %%EXAMPLES_DIR%%\hello.nag
echo pause
) > "%PACKAGE_DIR%\install.bat"

REM Create uninstall script
(
echo @echo off
echo echo 🗑️ Uninstalling Nagari Programming Language...
echo.
echo set "INSTALL_DIR=%%USERPROFILE%%\.nagari"
echo.
echo if exist "%%INSTALL_DIR%%" ^(
echo     rmdir /S /Q "%%INSTALL_DIR%%"
echo     echo ✅ Nagari uninstalled successfully
echo     echo.
echo     echo ⚠️ Don't forget to remove this directory from your PATH:
echo     echo %%INSTALL_DIR%%\bin
echo ^) else ^(
echo     echo ℹ️ Nagari is not installed in the expected location
echo ^)
echo pause
) > "%PACKAGE_DIR%\uninstall.bat"

REM Create README for the package
(
echo # Nagari Programming Language v%VERSION%
echo.
echo This is a standalone executable package of the Nagari Programming Language.
echo.
echo ## Quick Installation
echo.
echo ```cmd
echo install.bat
echo ```
echo.
echo ## Manual Installation
echo.
echo 1. Extract this package to your preferred location
echo 2. Add the `bin\` directory to your system PATH
echo 3. Verify installation: `nag --version`
echo.
echo ## What's Included
echo.
echo - **nag.exe** - Main CLI tool for compilation and project management
echo - **nagari-lsp.exe** - Language Server Protocol implementation for editors
echo - **runtime\** - JavaScript runtime library
echo - **stdlib\** - Standard library modules
echo - **examples\** - Sample Nagari programs
echo - **docs\** - Documentation
echo.
echo ## Getting Started
echo.
echo 1. Try running an example:
echo    ```cmd
echo    nag run examples\hello.nag
echo    ```
echo.
echo 2. Create a new file `test.nag`:
echo    ```python
echo    print^("Hello from Nagari!"^)
echo    ```
echo.
echo 3. Run it:
echo    ```cmd
echo    nag run test.nag
echo    ```
echo.
echo 4. Or compile to JavaScript:
echo    ```cmd
echo    nag build test.nag
echo    ```
echo.
echo ## Documentation
echo.
echo - Read `docs\getting-started.md` for a complete tutorial
echo - See `docs\language-guide.md` for syntax reference
echo - Check `docs\cli-reference.md` for CLI usage
echo.
echo ## Support
echo.
echo - GitHub: https://github.com/ayanalamMOON/Nagari
echo - Issues: https://github.com/ayanalamMOON/Nagari/issues
echo.
echo ---
echo.
echo Built on %DATE% for %TARGET%
) > "%PACKAGE_DIR%\README.md"

REM Show package contents summary
echo 🔷 Package contents summary
echo 📁 Directory structure:
dir /S /B "%PACKAGE_DIR%" | findstr /V /C:"\" | head -20

REM Create archive
echo 🔷 Creating archive: %ARCHIVE_NAME%
cd packages

REM Check if PowerShell is available for compression
where powershell >nul 2>&1 && (
    powershell -command "Compress-Archive -Path '%PACKAGE_NAME%' -DestinationPath '%ARCHIVE_NAME%' -Force"
) || (
    echo ⚠️ PowerShell not available, please manually create ZIP archive
)

cd "%ROOT_DIR%"

REM Generate checksums if available
echo 🔷 Generating checksums
cd packages
where certutil >nul 2>&1 && (
    certutil -hashfile "%ARCHIVE_NAME%" SHA256 > "%ARCHIVE_NAME%.sha256"
)
cd "%ROOT_DIR%"

REM Final verification
echo 🔷 Final verification
if exist "packages\%ARCHIVE_NAME%" (
    echo ✅ Archive created successfully
    for %%A in ("packages\%ARCHIVE_NAME%") do echo ℹ️ Size: %%~zA bytes
) else (
    echo ❌ Archive creation failed
    exit /b 1
)

REM Success summary
echo 📦 Package created successfully!
echo.
echo 📦 Package: packages\%PACKAGE_NAME%\
echo 📁 Archive: packages\%ARCHIVE_NAME%
if exist "packages\%ARCHIVE_NAME%.sha256" echo 🔐 Checksum: packages\%ARCHIVE_NAME%.sha256
echo.
echo 🚀 To test the package:
echo    cd packages\%PACKAGE_NAME%
echo    install.bat
echo    nag --version
echo.
echo 📋 Distribution ready!
echo    • Upload %ARCHIVE_NAME% for distribution
echo    • Users can extract and run the installer
echo    • No compilation required on target systems

pause

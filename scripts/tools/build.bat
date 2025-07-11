@echo off
rem Nagari Build Script for Windows
rem Builds the compiler, runtime, and all advanced features

echo Building Nagari Language Tools...

rem Build compiler
echo Building compiler...
cd nagari-compiler
cargo build --release
if %errorlevel% neq 0 exit /b %errorlevel%
cd ..

rem Build VM
echo Building VM...
cd nagari-vm
cargo build --release
if %errorlevel% neq 0 exit /b %errorlevel%
cd ..

rem Build runtime
echo Building runtime...
cd nagari-runtime
npm install
npm run build
if %errorlevel% neq 0 exit /b %errorlevel%
cd ..

rem Build CLI
echo Building CLI...
cd cli
cargo build --release
if %errorlevel% neq 0 exit /b %errorlevel%
cd ..

rem Build LSP server
echo Building LSP server...
cd lsp-server
cargo build --release
if %errorlevel% neq 0 exit /b %errorlevel%
cd ..

rem Build registry server
echo Building registry server...
cd registry-server
cargo build --release
if %errorlevel% neq 0 exit /b %errorlevel%
cd ..

rem Build advanced features
echo Building WebAssembly runtime...
call tools\build-wasm.bat

echo Building embedded runtime...
call tools\build-embedded.bat

echo Build completed successfully!
echo.
echo Core binaries located at:
echo   CLI Tool:        cli\target\release\nagari.exe
echo   Compiler:        nagari-compiler\target\release\nagc.exe
echo   VM:              nagari-vm\target\release\nagari-vm.exe
echo   LSP Server:      lsp-server\target\release\nagari-lsp.exe
echo   Registry Server: registry-server\target\release\nagari-registry.exe
echo   Runtime:         nagari-runtime\dist\
echo.
echo WebAssembly packages:
echo   Browser:   nagari-wasm\pkg\
echo   React:     nagari-wasm\pkg\react\
echo.
echo Embedded runtimes:
echo   Python:    nagari-embedded\target\wheels\
echo   Node.js:   nagari-embedded\target\release\
echo   C Library: nagari-embedded\target\release\nagari_embedded.lib
echo.
echo Add the CLI tool to your PATH to use globally.

@echo off
rem Nagari Build Script for Windows
rem Builds the compiler and runtime

echo Building Nagari Language Tools...

rem Build compiler
echo Building compiler...
cd nagari-compiler
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

echo Build completed successfully!
echo.
echo Binaries located at:
echo   Compiler: nagari-compiler\target\release\nagc.exe
echo   Runtime:  nagari-runtime\dist\
echo.
echo Add the compiler to your PATH to use globally.

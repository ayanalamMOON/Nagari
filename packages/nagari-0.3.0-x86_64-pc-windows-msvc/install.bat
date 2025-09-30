@echo off
echo Installing Nagari Programming Language...

set "INSTALL_DIR=%USERPROFILE%\.nagari"
set "BIN_DIR=%INSTALL_DIR%\bin"
set "STDLIB_DIR=%INSTALL_DIR%\stdlib"
set "RUNTIME_DIR=%INSTALL_DIR%\runtime"
set "EXAMPLES_DIR=%INSTALL_DIR%\examples"

if not exist "%BIN_DIR%" mkdir "%BIN_DIR%"
if not exist "%STDLIB_DIR%" mkdir "%STDLIB_DIR%"
if not exist "%RUNTIME_DIR%" mkdir "%RUNTIME_DIR%"
if not exist "%EXAMPLES_DIR%" mkdir "%EXAMPLES_DIR%"

echo Copying binaries...
copy bin\*.exe "%BIN_DIR%\"

echo Copying standard library...
if exist "stdlib" xcopy /E /I /Q stdlib "%STDLIB_DIR%"

echo Copying runtime...
if exist "nagari-runtime" xcopy /E /I /Q nagari-runtime "%RUNTIME_DIR%"

echo Copying examples...
if exist "examples" xcopy /E /I /Q examples "%EXAMPLES_DIR%"

echo Copying documentation...
copy *.md "%INSTALL_DIR%\" >nul 2>&1
if exist "docs" xcopy /E /I /Q docs "%INSTALL_DIR%\docs"

echo.
echo Nagari installed successfully!
echo.
echo Add to your PATH: %BIN_DIR%
echo Verify: nag --version
pause

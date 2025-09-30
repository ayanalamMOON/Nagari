@echo off
echo Uninstalling Nagari...

set "INSTALL_DIR=%USERPROFILE%\.nagari"

if exist "%INSTALL_DIR%" (
    rmdir /S /Q "%INSTALL_DIR%"
    echo Nagari uninstalled successfully
) else (
    echo Nagari is not installed
)
pause

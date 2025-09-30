@echo off
REM Nagari Development Server (Windows)
REM Hot-reload development server for rapid iteration

setlocal enabledelayedexpansion

REM Configuration
set "PROJECT_ROOT=%~dp0.."
set "PORT=3000"
set "WEBSOCKET_PORT=3001"

echo ================================
echo   Nagari Development Server
echo ================================
echo.

REM Check dependencies
echo [INFO] Checking dependencies...

where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Cargo not found. Please install Rust.
    pause
    exit /b 1
)

where cargo-watch >nul 2>&1
if %errorlevel% neq 0 (
    echo [INFO] Installing cargo-watch...
    cargo install cargo-watch
)

echo [SUCCESS] Dependencies check passed

REM Build project
echo [BUILD] Building project...
cd /d "%PROJECT_ROOT%"
cargo build
if %errorlevel% neq 0 (
    echo [ERROR] Build failed
    pause
    exit /b 1
)

echo [SUCCESS] Build completed successfully

REM Start file watcher
echo [INFO] Starting file watcher...
start "Cargo Watch" cargo watch --watch src --watch stdlib --watch Cargo.toml --clear --exec check --exec test --shell "echo Rebuilding... && cargo build"

REM Start file server
where python >nul 2>&1
if %errorlevel% equ 0 (
    echo [INFO] Starting file server on port %PORT%...
    start "File Server" python -m http.server %PORT%
    echo [SUCCESS] File server started on http://localhost:%PORT%
) else (
    echo [WARNING] Python not found. File server not started.
)

REM Create development dashboard
echo [INFO] Creating development dashboard...
(
echo ^<!DOCTYPE html^>
echo ^<html lang="en"^>
echo ^<head^>
echo     ^<meta charset="UTF-8"^>
echo     ^<meta name="viewport" content="width=device-width, initial-scale=1.0"^>
echo     ^<title^>Nagari Development Dashboard^</title^>
echo     ^<style^>
echo         body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 0; padding: 20px; background: #1e1e1e; color: #fff; }
echo         .header { text-align: center; margin-bottom: 30px; }
echo         .card { background: #2d2d2d; border-radius: 8px; padding: 20px; margin: 10px; border: 1px solid #404040; }
echo         .button { display: inline-block; padding: 8px 16px; background: #2196F3; color: white; text-decoration: none; border-radius: 4px; margin: 5px; }
echo     ^</style^>
echo ^</head^>
echo ^<body^>
echo     ^<div class="header"^>
echo         ^<h1^>🚀 Nagari Development Dashboard^</h1^>
echo         ^<p^>Real-time development environment status^</p^>
echo     ^</div^>
echo     ^<div class="card"^>
echo         ^<h3^>Quick Links^</h3^>
echo         ^<a href="/examples/" class="button"^>Examples^</a^>
echo         ^<a href="/samples/" class="button"^>Samples^</a^>
echo         ^<a href="/docs/" class="button"^>Documentation^</a^>
echo     ^</div^>
echo     ^<div class="card"^>
echo         ^<h3^>Development Status^</h3^>
echo         ^<p^>File watcher: Running^</p^>
echo         ^<p^>Build status: Check terminal for updates^</p^>
echo     ^</div^>
echo ^</body^>
echo ^</html^>
) > "%PROJECT_ROOT%\dev-dashboard.html"

echo [SUCCESS] Development dashboard created

echo.
echo [SUCCESS] Nagari Development Server is running!
echo.
echo Available URLs:
echo   • Dashboard:     http://localhost:%PORT%/dev-dashboard.html
echo   • Examples:      http://localhost:%PORT%/examples/
echo   • Samples:       http://localhost:%PORT%/samples/
echo   • Documentation: http://localhost:%PORT%/docs/
echo.
echo Services:
echo   • File Watcher:  Active (Cargo Watch)
echo   • File Server:   Active (Port %PORT%)
echo.
echo Press any key to stop the development server...
pause >nul

REM Cleanup
taskkill /F /FI "WINDOWTITLE eq Cargo Watch*" >nul 2>&1
taskkill /F /FI "WINDOWTITLE eq File Server*" >nul 2>&1

echo [SUCCESS] Development server stopped

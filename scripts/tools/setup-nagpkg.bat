@echo off
setlocal enabledelayedexpansion

REM Nagari Package Manager Setup Script (Windows)
REM This script sets up the nagpkg package manager infrastructure

echo 🚀 Setting up Nagari Package Manager (nagpkg)...

REM Configuration
if "%NAGARI_HOME%"=="" set NAGARI_HOME=%USERPROFILE%\.nagari
if "%NAGPKG_CACHE%"=="" set NAGPKG_CACHE=%NAGARI_HOME%\cache
if "%NAGPKG_REGISTRY%"=="" set NAGPKG_REGISTRY=https://registry.nagari.dev
set NAGPKG_CONFIG_FILE=%NAGARI_HOME%\nagpkg.toml

REM Create directories
echo 📁 Creating directory structure...
mkdir "%NAGARI_HOME%" 2>nul
mkdir "%NAGPKG_CACHE%\packages" 2>nul
mkdir "%NAGPKG_CACHE%\tarballs" 2>nul
mkdir "%NAGPKG_CACHE%\metadata" 2>nul
mkdir "%NAGPKG_CACHE%\temp" 2>nul
mkdir "%NAGARI_HOME%\config" 2>nul
mkdir "%NAGARI_HOME%\sessions" 2>nul
mkdir "%NAGARI_HOME%\logs" 2>nul

REM Create nagpkg configuration file
echo ⚙️  Creating nagpkg configuration...
(
echo # Nagari Package Manager Configuration
echo.
echo [registry]
echo # Default registry URL
echo default = "%NAGPKG_REGISTRY%"
echo.
echo # Alternative registries
echo [registry.sources]
echo # nagari = "https://registry.nagari.dev"
echo # local = "http://localhost:4873"
echo.
echo [cache]
echo # Cache directory
echo dir = "%NAGPKG_CACHE%"
echo.
echo # Cache settings
echo max_size_mb = 1024
echo max_age_days = 30
echo prune_on_startup = false
echo.
echo [security]
echo # Package integrity verification
echo verify_integrity = true
echo verify_signatures = false
echo.
echo # Trusted publishers
echo trusted_publishers = []
echo.
echo [network]
echo # Network timeout in seconds
echo timeout = 30
echo.
echo # Retry settings
echo max_retries = 3
echo retry_delay_ms = 1000
echo.
echo [publishing]
echo # Default access level for published packages
echo default_access = "public"
echo.
echo # Package validation settings
echo require_readme = true
echo require_license = true
echo require_description = true
echo.
echo [workspace]
echo # Workspace settings
echo auto_detect = true
echo hoist_dependencies = true
echo save_exact = false
echo.
echo # Dev dependency handling
echo install_dev_by_default = false
) > "%NAGPKG_CONFIG_FILE%"

REM Create example nagari.json template
echo 📦 Creating package template...
(
echo {
echo   "name": "my-nagari-package",
echo   "version": "1.0.0",
echo   "description": "A Nagari package",
echo   "main": "src/main.nag",
echo   "scripts": {
echo     "build": "nag build",
echo     "test": "nag test",
echo     "dev": "nag run --watch",
echo     "lint": "nag lint",
echo     "format": "nag format"
echo   },
echo   "keywords": [],
echo   "author": "",
echo   "license": "ISC",
echo   "dependencies": {},
echo   "devDependencies": {},
echo   "nagari": {
echo     "source_dir": "src",
echo     "output_dir": "dist",
echo     "target": "es2020",
echo     "module_format": "esm",
echo     "compiler_options": {
echo       "strict": true,
echo       "debug": false,
echo       "optimize": true,
echo       "emit_source_maps": true
echo     }
echo   }
echo }
) > "%NAGARI_HOME%\package-template.json"

REM Create .nagignore template
(
echo # Dependencies
echo node_modules/
echo .nagari-cache/
echo.
echo # Build output
echo dist/
echo build/
echo *.js.map
echo.
echo # Logs
echo *.log
echo npm-debug.log*
echo yarn-debug.log*
echo yarn-error.log*
echo.
echo # Environment variables
echo .env
echo .env.local
echo .env.development.local
echo .env.test.local
echo .env.production.local
echo.
echo # IDE/Editor files
echo .vscode/
echo .idea/
echo *.swp
echo *.swo
echo *~
echo.
echo # OS generated files
echo .DS_Store
echo .DS_Store?
echo ._*
echo .Spotlight-V100
echo .Trashes
echo ehthumbs.db
echo Thumbs.db
echo desktop.ini
echo.
echo # Temporary files
echo tmp/
echo temp/
echo *.tmp
echo *.temp
echo.
echo # Test coverage
echo coverage/
echo .nyc_output/
) > "%NAGARI_HOME%\nagignore-template"

REM Create initial cache metadata
echo 💾 Initializing cache metadata...
(
echo {
echo   "packages": {},
echo   "integrity_checks": {},
echo   "access_times": {}
echo }
) > "%NAGPKG_CACHE%\cache-metadata.json"

REM Set up logging configuration
echo 📊 Setting up logging...
(
echo [logging]
echo level = "info"
echo file = "%NAGARI_HOME%\logs\nagpkg.log"
echo max_size_mb = 10
echo max_files = 5
echo console_output = true
echo.
echo [logging.modules]
echo # Module-specific log levels
echo nagpkg = "info"
echo registry = "info"
echo resolver = "debug"
echo cache = "info"
) > "%NAGARI_HOME%\logging.toml"

REM Create registry client configuration
echo 🌐 Setting up registry client...
(
echo [registry]
echo url = "%NAGPKG_REGISTRY%"
echo timeout = 30
echo.
echo [auth]
echo # Authentication token ^(will be set by 'nag package login'^)
echo token = ""
echo.
echo [features]
echo # Registry feature support
echo search = true
echo publish = true
echo unpublish = true
echo deprecate = true
echo statistics = true
) > "%NAGARI_HOME%\registry.toml"

REM Create sample workspace configuration
echo 🏢 Creating workspace template...
mkdir "%NAGARI_HOME%\templates\workspace" 2>nul
(
echo {
echo   "name": "my-nagari-workspace",
echo   "version": "1.0.0",
echo   "private": true,
echo   "workspaces": [
echo     "packages/*",
echo     "apps/*"
echo   ],
echo   "scripts": {
echo     "build": "nag build --workspace",
echo     "test": "nag test --workspace",
echo     "lint": "nag lint --workspace",
echo     "format": "nag format --workspace"
echo   },
echo   "devDependencies": {},
echo   "nagari": {
echo     "workspace": {
echo       "hoist_dependencies": true,
echo       "parallel_builds": true,
echo       "shared_config": true
echo     }
echo   }
echo }
) > "%NAGARI_HOME%\templates\workspace\nagari-workspace.json"

REM Create development tools configuration
echo 🔧 Setting up development tools...
(
echo [formatter]
echo indent_size = 2
echo use_tabs = false
echo max_line_length = 100
echo trailing_commas = true
echo semicolons = false
echo.
echo [linter]
echo rules = "recommended"
echo max_warnings = 100
echo treat_warnings_as_errors = false
echo.
echo [compiler]
echo target = "es2020"
echo module_format = "esm"
echo source_maps = true
echo minify = false
echo.
echo [bundler]
echo entry_points = ["src/main.nag"]
echo output_dir = "dist"
echo format = "esm"
echo splitting = true
echo external = []
) > "%NAGARI_HOME%\tools.toml"

REM Create environment setup script
echo 🌍 Creating environment setup...
(
echo @echo off
echo REM Nagari Environment Setup
echo set NAGARI_HOME=%NAGARI_HOME%
echo set NAGPKG_CACHE=%NAGPKG_CACHE%
echo set NAGPKG_REGISTRY=%NAGPKG_REGISTRY%
echo.
echo REM Check if nag is available
echo where nag >nul 2>nul
echo if %%errorlevel%% neq 0 ^(
echo     echo Warning: 'nag' command not found in PATH
echo     echo Please install the Nagari CLI or add it to your PATH
echo ^)
echo.
echo echo Nagari environment loaded!
echo echo   NAGARI_HOME: %%NAGARI_HOME%%
echo echo   NAGPKG_CACHE: %%NAGPKG_CACHE%%
echo echo   NAGPKG_REGISTRY: %%NAGPKG_REGISTRY%%
) > "%NAGARI_HOME%\setup-env.bat"

REM Create cleanup script
echo 🧹 Creating cleanup script...
(
echo @echo off
echo REM Nagari Package Manager Cleanup Script
echo.
echo echo 🧹 Cleaning up Nagari package manager...
echo.
echo REM Clean cache
echo echo Cleaning package cache...
echo if exist "%NAGPKG_CACHE%\packages" rmdir /s /q "%NAGPKG_CACHE%\packages"
echo if exist "%NAGPKG_CACHE%\tarballs" rmdir /s /q "%NAGPKG_CACHE%\tarballs"
echo if exist "%NAGPKG_CACHE%\temp" rmdir /s /q "%NAGPKG_CACHE%\temp"
echo mkdir "%NAGPKG_CACHE%\packages" 2^>nul
echo mkdir "%NAGPKG_CACHE%\tarballs" 2^>nul
echo mkdir "%NAGPKG_CACHE%\temp" 2^>nul
echo.
echo REM Reset cache metadata
echo ^(
echo echo {
echo echo   "packages": {},
echo echo   "integrity_checks": {},
echo echo   "access_times": {}
echo echo }
echo ^) ^> "%NAGPKG_CACHE%\cache-metadata.json"
echo.
echo REM Clean old logs
echo echo Cleaning old logs...
echo forfiles /p "%NAGARI_HOME%\logs" /m *.log /d -7 /c "cmd /c del @path" 2^>nul
echo.
echo echo ✅ Cleanup completed!
) > "%NAGARI_HOME%\cleanup.bat"

REM Create health check script
echo 🏥 Creating health check script...
(
echo @echo off
echo REM Nagari Package Manager Health Check
echo.
echo echo 🏥 Nagari Package Manager Health Check
echo echo ======================================
echo.
echo REM Check directories
echo echo 📁 Directory structure:
echo if exist "%NAGARI_HOME%" ^(echo   ✅ %NAGARI_HOME%^) else ^(echo   ❌ %NAGARI_HOME% ^(missing^)^)
echo if exist "%NAGPKG_CACHE%" ^(echo   ✅ %NAGPKG_CACHE%^) else ^(echo   ❌ %NAGPKG_CACHE% ^(missing^)^)
echo if exist "%NAGPKG_CACHE%\packages" ^(echo   ✅ %NAGPKG_CACHE%\packages^) else ^(echo   ❌ %NAGPKG_CACHE%\packages ^(missing^)^)
echo if exist "%NAGPKG_CACHE%\tarballs" ^(echo   ✅ %NAGPKG_CACHE%\tarballs^) else ^(echo   ❌ %NAGPKG_CACHE%\tarballs ^(missing^)^)
echo if exist "%NAGPKG_CACHE%\metadata" ^(echo   ✅ %NAGPKG_CACHE%\metadata^) else ^(echo   ❌ %NAGPKG_CACHE%\metadata ^(missing^)^)
echo.
echo REM Check configuration files
echo echo.
echo echo ⚙️  Configuration files:
echo if exist "%NAGPKG_CONFIG_FILE%" ^(echo   ✅ %NAGPKG_CONFIG_FILE%^) else ^(echo   ❌ %NAGPKG_CONFIG_FILE% ^(missing^)^)
echo if exist "%NAGARI_HOME%\logging.toml" ^(echo   ✅ %NAGARI_HOME%\logging.toml^) else ^(echo   ❌ %NAGARI_HOME%\logging.toml ^(missing^)^)
echo if exist "%NAGARI_HOME%\registry.toml" ^(echo   ✅ %NAGARI_HOME%\registry.toml^) else ^(echo   ❌ %NAGARI_HOME%\registry.toml ^(missing^)^)
echo.
echo REM Check cache
echo echo.
echo echo 💾 Cache status:
echo if exist "%NAGPKG_CACHE%\cache-metadata.json" ^(
echo     echo   ✅ Cache metadata exists
echo ^) else ^(
echo     echo   ❌ Cache metadata missing
echo ^)
echo.
echo REM Check nag command
echo echo.
echo echo 🔧 CLI tool:
echo where nag ^>nul 2^>nul
echo if %%errorlevel%% equ 0 ^(
echo     echo   ✅ nag command available
echo ^) else ^(
echo     echo   ❌ nag command not found
echo ^)
echo.
echo echo.
echo echo Health check completed!
) > "%NAGARI_HOME%\health-check.bat"

REM Print installation summary
echo.
echo ✅ Nagari Package Manager setup completed!
echo.
echo 📍 Installation details:
echo   NAGARI_HOME: %NAGARI_HOME%
echo   Cache directory: %NAGPKG_CACHE%
echo   Registry: %NAGPKG_REGISTRY%
echo   Configuration: %NAGPKG_CONFIG_FILE%
echo.
echo 🚀 Next steps:
echo   1. Run environment setup: "%NAGARI_HOME%\setup-env.bat"
echo   2. Run health check: "%NAGARI_HOME%\health-check.bat"
echo   3. Initialize a project: nag package init
echo   4. Install packages: nag package install ^<package-name^>
echo.
echo 📚 Documentation:
echo   - Package manager guide: docs\nagpkg-design.md
echo   - CLI reference: nag package --help
echo   - Configuration: %NAGPKG_CONFIG_FILE%
echo.
echo 🆘 Support:
echo   - Cleanup: "%NAGARI_HOME%\cleanup.bat"
echo   - Health check: "%NAGARI_HOME%\health-check.bat"
echo   - Logs: %NAGARI_HOME%\logs\

pause

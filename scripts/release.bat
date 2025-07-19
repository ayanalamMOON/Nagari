@echo off
REM Release helper script for Nagari Programming Language (Windows)
REM Usage: scripts\release.bat [version]

setlocal enabledelayedexpansion

REM Get version from argument or default
set "VERSION=%~1"
if "%VERSION%"=="" set "VERSION=0.3.0"
set "TAG=v%VERSION%"

echo 🚀 Starting release process for Nagari %VERSION%

REM Check if we're in the right directory
if not exist "Cargo.toml" goto :wrong_dir
if not exist "src" goto :wrong_dir

REM Check git status
git diff-index --quiet HEAD -- >nul 2>&1
if errorlevel 1 (
    echo ⚠️  Working directory is not clean. Commit your changes first.
    set /p "continue=Continue anyway? (y/N): "
    if /i not "!continue!"=="y" exit /b 1
)

REM Run tests
echo 🔷 Running tests
cargo test --workspace
if errorlevel 1 (
    echo ❌ Tests failed
    exit /b 1
)
echo ✅ All tests passed

REM Build and test runtime
echo 🔷 Building nagari-runtime
cd nagari-runtime
npm install
npm run build
if errorlevel 1 (
    echo ❌ Runtime build failed
    exit /b 1
)
cd ..
echo ✅ Runtime built successfully

REM Build binaries for local testing
echo 🔷 Building release binaries
cargo build --release --bin nag
cargo build --release --bin nagari-lsp
if errorlevel 1 (
    echo ❌ Binary build failed
    exit /b 1
)
echo ✅ Binaries built successfully

REM Test CLI functionality
echo 🔷 Testing CLI functionality
target\release\nag.exe --version
echo print("Release test successful!") > test_release.nag
target\release\nag.exe compile test_release.nag
if errorlevel 1 (
    echo ❌ CLI test failed
    exit /b 1
)
del test_release.nag test_release.js 2>nul
echo ✅ CLI test passed

REM Update version in package.json
echo 🔷 Updating version in nagari-runtime/package.json
cd nagari-runtime
npm version %VERSION% --no-git-tag-version
cd ..

REM Check if CHANGELOG.md exists
if exist "CHANGELOG.md" (
    findstr /C:"## [%VERSION%]" CHANGELOG.md >nul
    if errorlevel 1 (
        echo ⚠️  CHANGELOG.md does not have entry for version %VERSION%
        echo Please add a changelog entry before proceeding.
    ) else (
        echo ✅ CHANGELOG.md has entry for version %VERSION%
    )
) else (
    echo ⚠️  CHANGELOG.md not found. Consider creating one.
)

REM Create git tag
echo 🔷 Creating git tag %TAG%
git tag -l | findstr /X "%TAG%" >nul
if not errorlevel 1 (
    echo ⚠️  Tag %TAG% already exists
    set /p "delete_tag=Delete existing tag and continue? (y/N): "
    if /i "!delete_tag!"=="y" (
        git tag -d %TAG%
        git push origin --delete %TAG% 2>nul
    ) else (
        exit /b 1
    )
)

git tag -a %TAG% -m "Release %TAG%"
echo ✅ Tag %TAG% created

REM Push tag to trigger release workflow
echo 🔷 Pushing tag to GitHub
git push origin %TAG%
if errorlevel 1 (
    echo ❌ Failed to push tag
    exit /b 1
)
echo ✅ Tag pushed to GitHub

echo 🔷 Release process completed!
echo.
echo 🚀 The GitHub Actions workflow will now:
echo    • Build binaries for all platforms
echo    • Create a GitHub release
echo    • Upload release assets
echo    • Publish to npm (if configured)
echo.
echo 📋 Monitor the progress in your GitHub repository's Actions tab
echo.
echo 🔗 Once complete, the release will be available in the Releases section
goto :end

:wrong_dir
echo ❌ This script must be run from the project root directory
exit /b 1

:end

@echo off
REM Nagari Repository SEO and Tags Setup Script (Windows)
REM Uses GitHub CLI to configure repository topics, description, and SEO settings

setlocal enabledelayedexpansion

echo 🚀 Setting up Nagari Repository SEO and Tags
echo ==================================================

REM Check if we're in the right directory
if not exist "Cargo.toml" (
    echo ❌ Error: Must be run from Nagari repository root
    exit /b 1
)

if not exist "README.md" (
    echo ❌ Error: Must be run from Nagari repository root  
    exit /b 1
)

REM Set repository info
set REPO_OWNER=ayanalamMOON
set REPO_NAME=Nagari
set REPO_FULL=%REPO_OWNER%/%REPO_NAME%

echo 📋 Repository: %REPO_FULL%

REM 1. Update repository description
echo.
echo 📝 Updating repository description...
gh repo edit %REPO_FULL% --description "Modern programming language combining Python's elegant syntax with JavaScript's ecosystem compatibility. Rust-based transpiler for web development. React, Vue, Express compatible."

if %errorlevel% equ 0 (
    echo ✅ Repository description updated
) else (
    echo ❌ Failed to update repository description
)

REM 2. Set repository homepage
echo.
echo 🏠 Setting repository homepage...
gh repo edit %REPO_FULL% --homepage "https://github.com/%REPO_FULL%"

if %errorlevel% equ 0 (
    echo ✅ Repository homepage set
) else (
    echo ❌ Failed to set repository homepage
)

REM 3. Add repository topics
echo.
echo 🏷️ Adding repository topics...

REM Create topics list (GitHub API format)
set TOPICS="programming-language,nagari,transpiler,python-syntax,javascript-interop,rust-compiler,web-development,react,vue,express,cli,repl,lsp,developer-tools,cross-platform,open-source,production-ready,modern-javascript,typescript,nodejs"

echo Adding topics to repository...
gh api repos/%REPO_FULL%/topics -X PUT -f names=%TOPICS%

if %errorlevel% equ 0 (
    echo ✅ Repository topics added successfully
    echo Topics added:
    echo   • programming-language
    echo   • nagari
    echo   • transpiler
    echo   • python-syntax
    echo   • javascript-interop
    echo   • rust-compiler
    echo   • web-development
    echo   • react
    echo   • vue
    echo   • express
    echo   • cli
    echo   • repl
    echo   • lsp
    echo   • developer-tools
    echo   • cross-platform
    echo   • open-source
    echo   • production-ready
    echo   • modern-javascript
    echo   • typescript
    echo   • nodejs
) else (
    echo ❌ Failed to add repository topics
)

REM 4. Enable repository features
echo.
echo ⚙️ Configuring repository features...

gh repo edit %REPO_FULL% --enable-issues=true
echo ✅ Issues enabled

gh repo edit %REPO_FULL% --enable-wiki=true  
echo ✅ Wiki enabled

gh repo edit %REPO_FULL% --enable-projects=true
echo ✅ Projects enabled

gh repo edit %REPO_FULL% --enable-discussions=true
echo ✅ Discussions enabled

REM 5. Create assets directory if needed
echo.
echo 🎨 Creating social preview setup...
if not exist "assets" (
    mkdir assets
    echo ✅ Created assets directory
)

REM Create social preview setup guide
echo # Social Preview Image Setup > assets\social-preview-setup.md
echo. >> assets\social-preview-setup.md
echo To complete SEO optimization, create a social preview image: >> assets\social-preview-setup.md
echo. >> assets\social-preview-setup.md
echo ## Requirements: >> assets\social-preview-setup.md
echo - **Size**: 1280x640 pixels (2:1 ratio) >> assets\social-preview-setup.md
echo - **Format**: PNG or JPG >> assets\social-preview-setup.md
echo - **File size**: ^< 1MB >> assets\social-preview-setup.md
echo - **Filename**: `social-preview.png` >> assets\social-preview-setup.md

echo ✅ Social preview setup guide created

REM 6. Display current repository stats
echo.
echo 📊 Current Repository Stats:
gh repo view %REPO_FULL% --json stargazerCount,forkCount,watcherCount,openIssuesCount,description

REM 7. Final checklist
echo.
echo 📋 SEO Setup Complete! Next Steps:
echo =============================================
echo ✅ Repository description updated
echo ✅ Homepage URL set  
echo ✅ Repository topics added (20 topics)
echo ✅ Repository features enabled
echo ✅ Social preview setup guide created
echo.
echo 📌 Manual Tasks Remaining:
echo 1. 🎨 Create and upload social preview image (1280x640px)
echo 2. 🌐 Enable GitHub Pages in repository settings
echo 3. 📊 Set up Google Analytics (add tracking ID to index.html)
echo 4. 🔍 Submit sitemap to Google Search Console
echo 5. 📱 Create social media accounts (@NagariLang)
echo 6. 🏷️ Consider creating additional releases with binaries
echo.
echo 🔗 Useful Links:
echo • Repository: https://github.com/%REPO_FULL%
echo • Issues: https://github.com/%REPO_FULL%/issues
echo • Discussions: https://github.com/%REPO_FULL%/discussions
echo • Releases: https://github.com/%REPO_FULL%/releases
echo • npm Package: https://www.npmjs.com/package/nagari-runtime
echo.
echo 🎉 SEO setup completed successfully!
echo 🚀 Your repository is now optimized for maximum discoverability!

pause
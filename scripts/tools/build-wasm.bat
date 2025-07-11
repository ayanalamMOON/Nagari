@echo off
echo Building Nagari WebAssembly Runtime...

REM Check for required tools
where wasm-pack >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo wasm-pack is required but not installed. Please install it from:
    echo https://rustwasm.github.io/wasm-pack/installer/
    exit /b 1
)

REM Build WebAssembly package
cd nagari-wasm
call wasm-pack build --target web --out-dir pkg --release

REM Build for different targets
echo Building for web target...
call wasm-pack build --target web --out-dir pkg/web --release

echo Building for Node.js target...
call wasm-pack build --target nodejs --out-dir pkg/nodejs --release

echo Building for bundler target...
call wasm-pack build --target bundler --out-dir pkg/bundler --release

echo Building for no-modules target...
call wasm-pack build --target no-modules --out-dir pkg/no-modules --release

REM Create package.json for npm publishing
(
echo {
echo   "name": "nagari-wasm",
echo   "version": "0.3.0",
echo   "description": "WebAssembly runtime for the Nagari programming language",
echo   "main": "nagari_wasm.js",
echo   "types": "nagari_wasm.d.ts",
echo   "files": [
echo     "nagari_wasm_bg.wasm",
echo     "nagari_wasm.js",
echo     "nagari_wasm.d.ts"
echo   ],
echo   "repository": {
echo     "type": "git",
echo     "url": "https://github.com/nagari-lang/nagari"
echo   },
echo   "keywords": [
echo     "webassembly",
echo     "wasm",
echo     "nagari",
echo     "programming-language",
echo     "runtime",
echo     "interpreter"
echo   ],
echo   "author": "Nagari Language Team",
echo   "license": "MIT",
echo   "homepage": "https://nagari.dev",
echo   "bugs": {
echo     "url": "https://github.com/nagari-lang/nagari/issues"
echo   }
echo }
) > pkg\package.json

echo Creating React integration package...
mkdir pkg\react 2>nul

(
echo {
echo   "name": "nagari-react",
echo   "version": "0.3.0",
echo   "description": "React integration for Nagari WebAssembly runtime",
echo   "main": "index.js",
echo   "types": "index.d.ts",
echo   "peerDependencies": {
echo     "react": ">=16.8.0"
echo   },
echo   "dependencies": {
echo     "nagari-wasm": "^0.3.0"
echo   },
echo   "repository": {
echo     "type": "git",
echo     "url": "https://github.com/nagari-lang/nagari"
echo   },
echo   "keywords": [
echo     "react",
echo     "nagari",
echo     "webassembly",
echo     "hooks"
echo   ],
echo   "author": "Nagari Language Team",
echo   "license": "MIT"
echo }
) > pkg\react\package.json

echo WebAssembly runtime build completed!
echo Files generated:
echo   - nagari-wasm\pkg\ - Main WebAssembly package
echo   - nagari-wasm\pkg\web\ - Web target
echo   - nagari-wasm\pkg\nodejs\ - Node.js target
echo   - nagari-wasm\pkg\bundler\ - Bundler target
echo   - nagari-wasm\pkg\no-modules\ - No modules target
echo   - nagari-wasm\pkg\react\ - React integration

cd ..

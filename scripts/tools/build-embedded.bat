@echo off
echo Building Nagari Embedded Runtime...

REM Build Rust library
cd nagari-embedded

echo Building core embedded runtime...
cargo build --release

echo Building Python bindings...
where python >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    cargo build --release --features python

    REM Check for maturin
    where maturin >nul 2>nul
    if %ERRORLEVEL% EQU 0 (
        maturin build --release --features python
        echo Python wheel built successfully
    ) else (
        echo maturin not found, skipping Python wheel creation
        echo Install with: pip install maturin
    )
) else (
    echo Python not found, skipping Python bindings
)

echo Building Node.js bindings...
where node >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    cargo build --release --features nodejs
    echo Node.js bindings built successfully
) else (
    echo Node.js not found, skipping Node.js bindings
)

echo Building C bindings...
cargo build --release --features c-bindings

REM Create C header file
(
echo #ifndef NAGARI_H
echo #define NAGARI_H
echo.
echo #include ^<stdint.h^>
echo #include ^<stddef.h^>
echo.
echo #ifdef __cplusplus
echo extern "C" {
echo #endif
echo.
echo // Forward declarations
echo typedef struct CNagariRuntime CNagariRuntime;
echo.
echo // Configuration structure
echo typedef struct CNagariConfig {
echo     size_t memory_limit;
echo     uint64_t execution_timeout;
echo     int allow_io;
echo     int allow_network;
echo     int sandbox_mode;
echo     int debug_mode;
echo } CNagariConfig;
echo.
echo // Value type enumeration
echo typedef enum CNagariValueType {
echo     NAGARI_VALUE_NONE = 0,
echo     NAGARI_VALUE_BOOL = 1,
echo     NAGARI_VALUE_INT = 2,
echo     NAGARI_VALUE_FLOAT = 3,
echo     NAGARI_VALUE_STRING = 4,
echo     NAGARI_VALUE_ARRAY = 5,
echo     NAGARI_VALUE_OBJECT = 6
echo } CNagariValueType;
echo.
echo // Runtime functions
echo CNagariRuntime* nagari_runtime_new^(const CNagariConfig* config^);
echo void nagari_runtime_destroy^(CNagariRuntime* runtime^);
echo.
echo #ifdef __cplusplus
echo }
echo #endif
echo.
echo #endif // NAGARI_H
) > target\release\nagari.h

echo Creating examples directory...
mkdir examples 2>nul

echo Embedded runtime build completed!
echo Generated files:
echo   - target\release\nagari_embedded.lib - Static library ^(Windows^)
echo   - target\release\nagari_embedded.dll - Dynamic library ^(Windows^)
echo   - target\release\nagari.h - C header file
echo   - examples\ - Example programs

cd ..

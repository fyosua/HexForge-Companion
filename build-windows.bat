@echo off
REM ============================================================================
REM HexForge Companion — Windows Build Script
REM Builds the .exe + NSIS installer for Windows x86_64
REM Requires: Rust, Node.js, Git
REM ============================================================================

echo === HexForge Companion — Windows Build ===
echo.

REM Check prerequisites
where rustc >nul 2>&1 || (echo ERROR: Rust not found. Install from https://rustup.rs/ && exit /b 1)
where node >nul 2>&1 || (echo ERROR: Node.js not found. Install from https://nodejs.org/ && exit /b 1)
where npm >nul 2>&1 || (echo ERROR: npm not found. && exit /b 1)

REM Install frontend deps
echo [1/4] Installing frontend dependencies...
call npm ci
if %ERRORLEVEL% neq 0 (echo ERROR: npm ci failed && exit /b %ERRORLEVEL%)

REM Install Tauri CLI
echo [2/4] Installing Tauri CLI...
call cargo install tauri-cli --version "^2"
if %ERRORLEVEL% neq 0 (echo ERROR: tauri-cli install failed && exit /b %ERRORLEVEL%)

REM Build the app
echo [3/4] Building HexForge Companion...
cd src-tauri
cargo tauri build --bundles nsis,msi --target x86_64-pc-windows-msvc
if %ERRORLEVEL% neq 0 (echo ERROR: Build failed && exit /b %ERRORLEVEL%)

REM Success
echo.
echo === BUILD COMPLETE ===
echo Installer: src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\
echo MSI:      src-tauri\target\x86_64-pc-windows-msvc\release\bundle\msi\
echo.
pause
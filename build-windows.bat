@echo off
REM Build script for Windows Server

echo.
echo Building SameSame Windows Server...
echo.

REM Check for Rust
where cargo >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Error: Rust is required but not installed.
    echo Please install from https://rustup.rs
    exit /b 1
)

REM Navigate to windows-server directory
cd /d "%~dp0\windows-server"

REM Build the server
echo Building release binary...
cargo build --release

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Build completed successfully!
    echo.
    echo The binary is located at:
    echo   %cd%\target\release\samesame-windows-server.exe
    echo.
    echo To run the server:
    echo   cd target\release
    echo   samesame-windows-server.exe
    echo.
) else (
    echo.
    echo Build failed!
    exit /b 1
)

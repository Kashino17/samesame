@echo off
REM Development script for Windows Server

echo.
echo Starting SameSame Windows Server in development mode...
echo.

REM Navigate to windows-server directory
cd /d "%~dp0\windows-server"

REM Run with debug logs
echo Starting server with debug logging...
set RUST_LOG=debug
cargo run

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Server stopped with error!
    pause
    exit /b 1
)

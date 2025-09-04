@echo off
echo Starting Guardian Update Server...
echo.

REM Check if Node.js is installed
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Node.js is not installed or not in PATH
    echo Please install Node.js from https://nodejs.org/
    pause
    exit /b 1
)

REM Install dependencies if needed
if not exist node_modules (
    echo Installing dependencies...
    npm install express
)

REM Start the update server
echo Starting update server on http://localhost:3000
echo Press Ctrl+C to stop the server
echo.
node update-server.js

pause

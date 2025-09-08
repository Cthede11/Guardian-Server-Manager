@echo off
echo Starting Guardian with Backend...

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%..

REM Start the backend first
echo Starting backend...
start /B "" "%PROJECT_ROOT%\build\executables\hostd.exe"

REM Wait a moment for backend to start
timeout /t 3 /nobreak >nul

REM Check if backend is running
echo Checking backend status...
curl -s http://localhost:8080/health >nul 2>&1
if %errorlevel% equ 0 (
    echo Backend is running successfully
) else (
    echo Warning: Backend may not be running properly
)

REM Start the Guardian UI
echo Starting Guardian UI...
cd /d "%PROJECT_ROOT%\guardian-ui"
npm run tauri dev

pause
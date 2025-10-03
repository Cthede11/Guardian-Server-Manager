@echo off
echo Starting Guardian with Backend...

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%..

REM Start the backend first
echo Starting backend...
set BACKEND_PATH=%PROJECT_ROOT%\build\executables\hostd.exe

REM Use start /B for background processes
start /B "" "%BACKEND_PATH%"

REM Wait a moment for backend to start
timeout /t 3 /nobreak >nul

REM Check if backend is running
echo Checking backend status...
powershell -Command "try { $Response = Invoke-WebRequest -Uri 'http://localhost:8080/health' -Method GET -TimeoutSec 5; if ($Response.StatusCode -eq 200) { Write-Host 'Backend is running successfully' -ForegroundColor Green } else { Write-Host 'Warning: Backend responded with status' $Response.StatusCode -ForegroundColor Yellow } } catch { Write-Host 'Warning: Backend may not be running properly:' $_.Exception.Message -ForegroundColor Red }"

REM Start the Guardian UI
echo Starting Guardian UI...
cd /d "%PROJECT_ROOT%\guardian-ui"

REM Start the UI and capture the process ID
start "" npm run tauri dev
set UI_PID=%ERRORLEVEL%

REM Wait for user to close the application
echo Guardian is running. Press any key to stop...
pause >nul

REM Cleanup processes
echo Cleaning up processes...
taskkill /F /IM hostd.exe >nul 2>&1
taskkill /F /IM guardian.exe >nul 2>&1
echo Cleanup completed

echo Guardian stopped.

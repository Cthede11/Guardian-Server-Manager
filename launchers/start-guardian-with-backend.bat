@echo off
echo Starting Guardian with Backend Services...
echo.

REM Check if database exists in project root, create if not
if not exist "data\guardian.db" (
    echo Creating database...
    mkdir data 2>nul
    echo. > data\guardian.db
)

REM Change to the release directory
cd /d "%~dp0guardian-ui\src-tauri\target\release"

REM Start the backend service in the background
echo Starting backend service (hostd)...
start /B "%~dp0hostd\target\release\hostd.exe" --port 8080 --log-level info --config "%~dp0configs\hostd.yaml"

REM Wait a moment for the backend to start
timeout /t 3 /nobreak >nul

REM Check if hostd is running
tasklist /FI "IMAGENAME eq hostd.exe" 2>NUL | find /I /N "hostd.exe">NUL
if "%ERRORLEVEL%"=="0" (
    echo ✅ Backend service started successfully
) else (
    echo ❌ Backend service failed to start
)

REM Start the Guardian application
echo Starting Guardian application...
guardian.exe

REM Clean up - kill hostd when Guardian closes
echo Cleaning up backend service...
taskkill /F /IM hostd.exe 2>nul

echo Guardian has been closed.
pause

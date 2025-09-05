@echo off
echo 🚀 Launching Guardian Minecraft Server Manager...
echo ===============================================

REM Check if we're in the right directory
if not exist "guardian-ui" (
    echo ❌ Please run this script from the project root directory
    pause
    exit /b 1
)

REM Try to run the Tauri application first
if exist "guardian-ui\src-tauri\target\release\guardian.exe" (
    echo ✅ Found Guardian executable, launching...
    start "" "guardian-ui\src-tauri\target\release\guardian.exe"
    echo 🎯 Guardian launched successfully!
    pause
    exit /b 0
)

REM If Tauri app not found, try running backend and frontend separately
echo ⚠️  Tauri app not found, trying alternative launch method...

REM Check if backend exists
if exist "hostd\target\release\hostd.exe" (
    echo ✅ Found backend executable
    
    REM Start backend in background
    echo 🚀 Starting backend...
    start /min "" "hostd\target\release\hostd.exe" --config "configs\hostd.yaml" --port 8080
    
    REM Wait a moment for backend to start
    timeout /t 3 /nobreak >nul
    
    REM Start frontend
    echo 🎨 Starting frontend...
    cd guardian-ui
    npm run dev
    
    echo 🎯 Guardian launched! Open your browser to http://localhost:8080
    pause
) else (
    echo ❌ Backend executable not found. Please build the application first.
    echo Run: .\build-simple.ps1
    pause
    exit /b 1
)
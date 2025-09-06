@echo off
title Guardian Shortcut Setup
echo.
echo ========================================
echo   Guardian Shortcut Setup
echo ========================================
echo.
echo This will configure your Guardian shortcuts to launch with backend services.
echo.

REM Check if we're in the installation directory
if not exist "guardian.exe" (
    echo Error: Please run this script from the Guardian installation directory.
    echo Usually: C:\Program Files\Guardian - Minecraft Server Manager
    echo.
    pause
    exit /b 1
)

echo Found Guardian installation. Setting up shortcuts...
echo.

REM Run the PowerShell setup script
powershell -ExecutionPolicy Bypass -File "post-install-setup.ps1"

echo.
echo Setup complete! Your Guardian shortcuts now launch with backend services.
echo.
echo You can now:
echo   - Double-click the desktop shortcut to start Guardian with backend
echo   - Use the Start Menu shortcuts for different launch options
echo   - Create servers without any "Failed to fetch" errors
echo.
pause

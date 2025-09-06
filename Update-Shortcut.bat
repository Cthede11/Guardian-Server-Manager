@echo off
title Update Guardian Desktop Shortcut
echo.
echo ========================================
echo   Update Guardian Desktop Shortcut
echo ========================================
echo.
echo This will update your desktop shortcut to point to the launcher.
echo.

REM Run the PowerShell script
powershell -ExecutionPolicy Bypass -File "update-desktop-shortcut.ps1"

echo.
echo Press any key to exit...
pause >nul

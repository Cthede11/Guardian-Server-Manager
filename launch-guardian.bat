@echo off
title Guardian - Minecraft Server Manager
echo.
echo ========================================
echo   Guardian - Minecraft Server Manager
echo ========================================
echo.
echo Starting Guardian with Backend Services...
echo.

REM Run the PowerShell launcher script
powershell -ExecutionPolicy Bypass -File "launchers\start-guardian-with-backend.ps1"

echo.
echo Press any key to exit...
pause >nul
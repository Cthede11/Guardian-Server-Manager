# Post-Installation Setup Script for Guardian
# This script modifies the shortcuts created by the installer to point to the launcher

Write-Host "Setting up Guardian shortcuts to launch with backend..." -ForegroundColor Green

# Get the installation directory
$installDir = "${env:ProgramFiles}\Guardian - Minecraft Server Manager"
if (-not (Test-Path $installDir)) {
    $installDir = "${env:ProgramFiles(x86)}\Guardian - Minecraft Server Manager"
}

if (-not (Test-Path $installDir)) {
    Write-Host "Guardian installation not found. Please run this script after installing Guardian." -ForegroundColor Red
    exit 1
}

Write-Host "Found Guardian installation at: $installDir" -ForegroundColor Cyan

# Update desktop shortcut
$desktopShortcut = "$env:USERPROFILE\Desktop\Guardian - Minecraft Server Manager.lnk"
if (Test-Path $desktopShortcut) {
    Write-Host "Updating desktop shortcut..." -ForegroundColor Yellow
    
    $WshShell = New-Object -comObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($desktopShortcut)
    $Shortcut.TargetPath = "$installDir\start-guardian-with-backend.bat"
    $Shortcut.WorkingDirectory = $installDir
    $Shortcut.Description = "Guardian - Minecraft Server Manager with Backend Services"
    $Shortcut.Save()
    
    Write-Host "Desktop shortcut updated" -ForegroundColor Green
}

# Update start menu shortcuts
$startMenuDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Guardian - Minecraft Server Manager"
if (Test-Path $startMenuDir) {
    Write-Host "Updating start menu shortcuts..." -ForegroundColor Yellow
    
    # Update main shortcut
    $mainShortcut = "$startMenuDir\Guardian - Minecraft Server Manager.lnk"
    if (Test-Path $mainShortcut) {
        $WshShell = New-Object -comObject WScript.Shell
        $Shortcut = $WshShell.CreateShortcut($mainShortcut)
        $Shortcut.TargetPath = "$installDir\start-guardian-with-backend.bat"
        $Shortcut.WorkingDirectory = $installDir
        $Shortcut.Description = "Guardian - Minecraft Server Manager with Backend Services"
        $Shortcut.Save()
    }
    
    # Create PowerShell launcher shortcut
    $psShortcut = "$startMenuDir\Guardian (PowerShell).lnk"
    $WshShell = New-Object -comObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($psShortcut)
    $Shortcut.TargetPath = "powershell.exe"
    $Shortcut.Arguments = "-ExecutionPolicy Bypass -File `"$installDir\start-guardian-with-backend.ps1`""
    $Shortcut.WorkingDirectory = $installDir
    $Shortcut.Description = "Guardian - Minecraft Server Manager (PowerShell Launcher)"
    $Shortcut.IconLocation = "$installDir\guardian.exe,0"
    $Shortcut.Save()
    
    Write-Host "Start menu shortcuts updated" -ForegroundColor Green
}

Write-Host ""
Write-Host "✅ Setup complete!" -ForegroundColor Green
Write-Host "Guardian shortcuts now launch with backend services automatically." -ForegroundColor Cyan
Write-Host ""
Write-Host "You can now:" -ForegroundColor Yellow
Write-Host "  - Double-click the desktop shortcut to start Guardian with backend" -ForegroundColor White
Write-Host "  - Use the Start Menu shortcuts for different launch options" -ForegroundColor White
Write-Host "  - Create servers without any 'Failed to fetch' errors" -ForegroundColor White

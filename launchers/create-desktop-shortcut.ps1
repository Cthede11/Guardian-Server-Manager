# Create Desktop Shortcut for Guardian Launcher
Write-Host "Creating desktop shortcut for Guardian..." -ForegroundColor Green

# Get the current script directory (project root)
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$launcherPath = Join-Path $scriptPath "Launch-Guardian.bat"

# Get desktop path
$desktopPath = [Environment]::GetFolderPath("Desktop")
$shortcutPath = Join-Path $desktopPath "Guardian - Minecraft Server Manager.lnk"

# Create WScript.Shell object
$WshShell = New-Object -comObject WScript.Shell

# Create shortcut
$Shortcut = $WshShell.CreateShortcut($shortcutPath)
$Shortcut.TargetPath = $launcherPath
$Shortcut.WorkingDirectory = $scriptPath
$Shortcut.Description = "Guardian - Minecraft Server Manager with Backend Services"
$Shortcut.IconLocation = "shell32.dll,1"  # Use a default icon, you can change this later

# Save the shortcut
$Shortcut.Save()

Write-Host "Desktop shortcut created successfully!" -ForegroundColor Green
Write-Host "Shortcut location: $shortcutPath" -ForegroundColor Cyan
Write-Host "Target: $launcherPath" -ForegroundColor Cyan

# Test if the shortcut was created
if (Test-Path $shortcutPath) {
    Write-Host "✅ Shortcut verification: SUCCESS" -ForegroundColor Green
} else {
    Write-Host "❌ Shortcut verification: FAILED" -ForegroundColor Red
}

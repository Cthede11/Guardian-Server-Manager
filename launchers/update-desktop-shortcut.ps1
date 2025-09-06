# Update Existing Desktop Shortcut to Point to Launcher
Write-Host "Updating desktop shortcut to point to launcher..." -ForegroundColor Green

# Get the current script directory (project root)
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$launcherPath = Join-Path $scriptPath "Launch-Guardian.bat"

# Get desktop path
$desktopPath = [Environment]::GetFolderPath("Desktop")
$shortcutPath = Join-Path $desktopPath "Guardian - Minecraft Server Manager.lnk"

# Check if shortcut exists
if (Test-Path $shortcutPath) {
    Write-Host "Found existing shortcut, updating..." -ForegroundColor Yellow
    
    # Create WScript.Shell object
    $WshShell = New-Object -comObject WScript.Shell
    
    # Update shortcut
    $Shortcut = $WshShell.CreateShortcut($shortcutPath)
    $Shortcut.TargetPath = $launcherPath
    $Shortcut.WorkingDirectory = $scriptPath
    $Shortcut.Description = "Guardian - Minecraft Server Manager with Backend Services"
    $Shortcut.IconLocation = "shell32.dll,1"
    
    # Save the shortcut
    $Shortcut.Save()
    
    Write-Host "Desktop shortcut updated successfully!" -ForegroundColor Green
    Write-Host "Shortcut now points to: $launcherPath" -ForegroundColor Cyan
} else {
    Write-Host "No existing shortcut found. Creating new one..." -ForegroundColor Yellow
    
    # Create WScript.Shell object
    $WshShell = New-Object -comObject WScript.Shell
    
    # Create shortcut
    $Shortcut = $WshShell.CreateShortcut($shortcutPath)
    $Shortcut.TargetPath = $launcherPath
    $Shortcut.WorkingDirectory = $scriptPath
    $Shortcut.Description = "Guardian - Minecraft Server Manager with Backend Services"
    $Shortcut.IconLocation = "shell32.dll,1"
    
    # Save the shortcut
    $Shortcut.Save()
    
    Write-Host "New desktop shortcut created!" -ForegroundColor Green
    Write-Host "Shortcut location: $shortcutPath" -ForegroundColor Cyan
}

# Test if the shortcut was created/updated
if (Test-Path $shortcutPath) {
    Write-Host "✅ Shortcut verification: SUCCESS" -ForegroundColor Green
} else {
    Write-Host "❌ Shortcut verification: FAILED" -ForegroundColor Red
}

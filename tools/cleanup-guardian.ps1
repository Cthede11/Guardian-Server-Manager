# Guardian Complete Cleanup Script
# This script removes all traces of Guardian from the system

Write-Host "🧹 Guardian Complete Cleanup Script" -ForegroundColor Red
Write-Host "=====================================" -ForegroundColor Red
Write-Host "WARNING: This will remove ALL Guardian files and registry entries!" -ForegroundColor Yellow
Write-Host ""

# Function to safely remove registry entries
function Remove-RegistryEntry {
    param($Path, $Name)
    try {
        if (Test-Path $Path) {
            Remove-ItemProperty -Path $Path -Name $Name -ErrorAction SilentlyContinue
            Write-Host "✅ Removed registry entry: $Path\$Name" -ForegroundColor Green
        }
    } catch {
        Write-Host "⚠️  Could not remove registry entry: $Path\$Name" -ForegroundColor Yellow
    }
}

# Function to safely remove directories
function Remove-DirectorySafely {
    param($Path)
    try {
        if (Test-Path $Path) {
            Remove-Item -Path $Path -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host "✅ Removed directory: $Path" -ForegroundColor Green
        }
    } catch {
        Write-Host "⚠️  Could not remove directory: $Path" -ForegroundColor Yellow
    }
}

Write-Host "🔍 Step 1: Searching for Guardian registry entries..." -ForegroundColor Cyan

# Check both 64-bit and 32-bit registry locations
$registryPaths = @(
    "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"
)

$guardianKeys = @()

foreach ($regPath in $registryPaths) {
    if (Test-Path $regPath) {
        $keys = Get-ChildItem -Path $regPath -ErrorAction SilentlyContinue
        foreach ($key in $keys) {
            $properties = Get-ItemProperty -Path $key.PSPath -ErrorAction SilentlyContinue
            if ($properties -and ($properties.DisplayName -like "*Guardian*" -or $properties.Publisher -like "*Guardian*")) {
                $guardianKeys += $key
                Write-Host "Found Guardian entry: $($properties.DisplayName) at $($key.PSPath)" -ForegroundColor Yellow
            }
        }
    }
}

# Remove found registry entries
Write-Host "`n🗑️  Step 2: Removing registry entries..." -ForegroundColor Cyan
foreach ($key in $guardianKeys) {
    try {
        Remove-Item -Path $key.PSPath -Recurse -Force
        Write-Host "✅ Removed registry key: $($key.PSPath)" -ForegroundColor Green
    } catch {
        Write-Host "⚠️  Could not remove registry key: $($key.PSPath)" -ForegroundColor Yellow
    }
}

Write-Host "`n📁 Step 3: Removing Guardian directories..." -ForegroundColor Cyan

# Common Guardian installation locations
$guardianPaths = @(
    "$env:LOCALAPPDATA\Guardian",
    "$env:PROGRAMFILES\Guardian",
    "$env:PROGRAMFILES(X86)\Guardian",
    "$env:APPDATA\Guardian",
    "$env:USERPROFILE\AppData\Local\Guardian",
    "$env:USERPROFILE\AppData\Roaming\Guardian"
)

foreach ($path in $guardianPaths) {
    Remove-DirectorySafely $path
}

Write-Host "`n🔍 Step 4: Searching for remaining Guardian files..." -ForegroundColor Cyan

# Search for any remaining Guardian files
$searchPaths = @(
    "$env:LOCALAPPDATA",
    "$env:PROGRAMFILES",
    "$env:PROGRAMFILES(X86)",
    "$env:APPDATA"
)

foreach ($searchPath in $searchPaths) {
    if (Test-Path $searchPath) {
        $guardianFiles = Get-ChildItem -Path $searchPath -Name "*Guardian*" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 10
        if ($guardianFiles) {
            Write-Host "Found Guardian files in $searchPath :" -ForegroundColor Yellow
            foreach ($file in $guardianFiles) {
                Write-Host "  - $file" -ForegroundColor Yellow
            }
        }
    }
}

Write-Host "`n🧹 Step 5: Cleaning temporary files..." -ForegroundColor Cyan

# Clean temporary files
$tempPaths = @(
    "$env:TEMP\*Guardian*",
    "$env:TMP\*Guardian*"
)

foreach ($tempPath in $tempPaths) {
    Get-ChildItem -Path $tempPath -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "`n🔍 Step 6: Verifying cleanup..." -ForegroundColor Cyan

# Check if any Guardian entries still exist
$remainingEntries = @()
foreach ($regPath in $registryPaths) {
    if (Test-Path $regPath) {
        $keys = Get-ChildItem -Path $regPath -ErrorAction SilentlyContinue
        foreach ($key in $keys) {
            $properties = Get-ItemProperty -Path $key.PSPath -ErrorAction SilentlyContinue
            if ($properties -and ($properties.DisplayName -like "*Guardian*" -or $properties.Publisher -like "*Guardian*")) {
                $remainingEntries += $properties.DisplayName
            }
        }
    }
}

if ($remainingEntries.Count -eq 0) {
    Write-Host "✅ No Guardian registry entries found" -ForegroundColor Green
} else {
    Write-Host "⚠️  Remaining Guardian entries found:" -ForegroundColor Yellow
    foreach ($entry in $remainingEntries) {
        Write-Host "  - $entry" -ForegroundColor Yellow
    }
}

Write-Host "`n🎉 Guardian cleanup completed!" -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Green
Write-Host "You can now run the Guardian installer fresh." -ForegroundColor Cyan
Write-Host ""

# Ask if user wants to run the installer
$response = Read-Host "Would you like to run the Guardian installer now? (y/n)"
if ($response -eq "y" -or $response -eq "Y") {
    $installerPath = "guardian-ui\src-tauri\target\release\bundle\nsis\Guardian_1.0.0_x64-setup.exe"
    if (Test-Path $installerPath) {
        Write-Host "🚀 Starting Guardian installer..." -ForegroundColor Cyan
        Start-Process -FilePath $installerPath -Wait
    } else {
        Write-Host "❌ Installer not found at: $installerPath" -ForegroundColor Red
    }
}

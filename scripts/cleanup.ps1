# Cleanup script for Guardian project
# This script removes temporary files, build artifacts, and organizes the project structure

Write-Host "[CLEANUP] Starting Guardian project cleanup..." -ForegroundColor Green

# Set error action preference
$ErrorActionPreference = "Stop"

# Get the project root directory
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Write-Host "Project root: $ProjectRoot" -ForegroundColor Blue

# Clean up temporary files
Write-Host "[CLEANUP] Removing temporary files..." -ForegroundColor Yellow

# Remove common temporary files
$TempPatterns = @(
    "*.tmp",
    "*.temp", 
    "*.bak",
    "*.backup",
    "*.log",
    "*.swp",
    "*.swo",
    "*~"
)

foreach ($Pattern in $TempPatterns) {
    Get-ChildItem -Path $ProjectRoot -Recurse -Name $Pattern -File | 
        Where-Object { $_ -notlike "*target*" -and $_ -notlike "*node_modules*" -and $_ -notlike "*build/logs*" } |
        ForEach-Object {
            $FilePath = Join-Path $ProjectRoot $_
            Remove-Item $FilePath -Force -ErrorAction SilentlyContinue
            Write-Host "Removed: $_" -ForegroundColor Gray
        }
}

# Clean up build artifacts from source directories
Write-Host "[CLEANUP] Cleaning build artifacts from source directories..." -ForegroundColor Yellow

# Remove executables from source directories
$SourceDirs = @(
    "guardian-ui/src-tauri",
    "hostd",
    "gpu-worker"
)

foreach ($Dir in $SourceDirs) {
    $FullPath = Join-Path $ProjectRoot $Dir
    if (Test-Path $FullPath) {
        Get-ChildItem -Path $FullPath -Name "*.exe" -File | ForEach-Object {
            $FilePath = Join-Path $FullPath $_
            Remove-Item $FilePath -Force -ErrorAction SilentlyContinue
            Write-Host "Removed executable: $Dir/$_" -ForegroundColor Gray
        }
    }
}

# Clean up empty directories
Write-Host "[CLEANUP] Removing empty directories..." -ForegroundColor Yellow
Get-ChildItem -Path $ProjectRoot -Recurse -Directory | 
    Where-Object { (Get-ChildItem $_.FullName -Force | Measure-Object).Count -eq 0 } |
    ForEach-Object {
        Remove-Item $_.FullName -Force -ErrorAction SilentlyContinue
        Write-Host "Removed empty directory: $($_.Name)" -ForegroundColor Gray
    }

# Organize build artifacts
Write-Host "[CLEANUP] Organizing build artifacts..." -ForegroundColor Yellow

# Ensure build directory structure exists
$BuildDirs = @(
    "build/executables",
    "build/installers", 
    "build/temp",
    "build/logs"
)

foreach ($Dir in $BuildDirs) {
    $FullPath = Join-Path $ProjectRoot $Dir
    New-Item -ItemType Directory -Path $FullPath -Force | Out-Null
}

# Move any remaining executables to build/executables
$Executables = Get-ChildItem -Path $ProjectRoot -Recurse -Filter "*.exe" -File |
    Where-Object { $_.FullName -notlike "*target*" -and $_.FullName -notlike "*node_modules*" -and $_.FullName -notlike "*build*" }

foreach ($Exe in $Executables) {
    $SourcePath = $Exe.FullName
    $DestPath = Join-Path (Join-Path $ProjectRoot "build/executables") $Exe.Name
    Move-Item $SourcePath $DestPath -Force -ErrorAction SilentlyContinue
    Write-Host "Moved executable: $($Exe.Name) -> build/executables/" -ForegroundColor Gray
}

# Move any remaining installer scripts to build/installers
$InstallerPatterns = @("*.nsi", "*.wxs", "*.iss")
foreach ($Pattern in $InstallerPatterns) {
    $Installers = Get-ChildItem -Path $ProjectRoot -Recurse -Filter $Pattern -File |
        Where-Object { $_.FullName -notlike "*target*" -and $_.FullName -notlike "*node_modules*" -and $_.FullName -notlike "*build*" }
    
    foreach ($Installer in $Installers) {
        $SourcePath = $Installer.FullName
        $DestPath = Join-Path (Join-Path $ProjectRoot "build/installers") $Installer.Name
        Move-Item $SourcePath $DestPath -Force -ErrorAction SilentlyContinue
        Write-Host "Moved installer: $($Installer.Name) -> build/installers/" -ForegroundColor Gray
    }
}

# Move any remaining log files to build/logs
$Logs = Get-ChildItem -Path $ProjectRoot -Recurse -Filter "*.log" -File |
    Where-Object { $_.FullName -notlike "*target*" -and $_.FullName -notlike "*node_modules*" -and $_.FullName -notlike "*build*" }

foreach ($Log in $Logs) {
    $SourcePath = $Log.FullName
    $DestPath = Join-Path (Join-Path $ProjectRoot "build/logs") $Log.Name
    Move-Item $SourcePath $DestPath -Force -ErrorAction SilentlyContinue
    Write-Host "Moved log: $($Log.Name) -> build/logs/" -ForegroundColor Gray
}

Write-Host "[COMPLETE] Cleanup completed!" -ForegroundColor Green
Write-Host "[INFO] Build artifacts are now organized in the build/ directory" -ForegroundColor Blue
Write-Host "[INFO] - Executables: build/executables/" -ForegroundColor Blue
Write-Host "[INFO] - Installers: build/installers/" -ForegroundColor Blue
Write-Host "[INFO] - Logs: build/logs/" -ForegroundColor Blue

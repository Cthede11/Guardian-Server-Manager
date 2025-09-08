# Master build script for Guardian project
# This script builds all components and organizes everything in the build/ directory

Write-Host "[BUILD] Building Guardian project..." -ForegroundColor Green

# Set error action preference
$ErrorActionPreference = "Stop"

# Get the project root directory
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Write-Host "Project root: $ProjectRoot" -ForegroundColor Blue

# Run cleanup first
Write-Host "[CLEANUP] Running cleanup..." -ForegroundColor Yellow
& "$PSScriptRoot/cleanup.ps1"

# Build all components
Write-Host "[BUILD] Building all components..." -ForegroundColor Yellow
& "$PSScriptRoot/build-desktop.ps1"

# Final organization
Write-Host "[ORGANIZE] Final organization..." -ForegroundColor Yellow

# Ensure all build directories exist
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

# Copy any remaining executables
Get-ChildItem -Path $ProjectRoot -Recurse -Name "*.exe" -File |
    Where-Object { $_ -notlike "*target*" -and $_ -notlike "*node_modules*" -and $_ -notlike "*build*" } |
    ForEach-Object {
        $SourcePath = Join-Path $ProjectRoot $_
        $DestPath = Join-Path $ProjectRoot "build/executables" (Split-Path $_ -Leaf)
        if (-not (Test-Path $DestPath)) {
            Copy-Item $SourcePath $DestPath -Force -ErrorAction SilentlyContinue
            Write-Host "Copied executable: $_ -> build/executables/" -ForegroundColor Gray
        }
    }

Write-Host "[COMPLETE] Guardian project build completed!" -ForegroundColor Green
Write-Host "[OUTPUT] All build artifacts are organized in the build/ directory" -ForegroundColor Blue
Write-Host "[INFO] - Executables: build/executables/" -ForegroundColor Blue
Write-Host "[INFO] - Installers: build/installers/" -ForegroundColor Blue
Write-Host "[INFO] - Logs: build/logs/" -ForegroundColor Blue

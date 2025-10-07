# Guardian Production Build Script
# This script builds all components with proper error handling and artifact organization

param(
    [switch]$Clean,
    [switch]$SkipTests,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

Write-Host "=== GUARDIAN PRODUCTION BUILD ===" -ForegroundColor Green
Write-Host "Project Root: $ProjectRoot" -ForegroundColor Cyan

# Function to log with timestamp
function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $Color = switch ($Level) {
        "ERROR" { "Red" }
        "WARN" { "Yellow" }
        "SUCCESS" { "Green" }
        default { "White" }
    }
    Write-Host "[$Timestamp] [$Level] $Message" -ForegroundColor $Color
}

# Function to check if command exists
function Test-Command {
    param([string]$Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    } catch {
        return $false
    }
}

# Check prerequisites
Write-Log "Checking prerequisites..."
$Prerequisites = @("cargo", "npm", "node")
foreach ($cmd in $Prerequisites) {
    if (-not (Test-Command $cmd)) {
        Write-Log "Missing prerequisite: $cmd" "ERROR"
        exit 1
    }
}
Write-Log "All prerequisites found" "SUCCESS"

# Clean build if requested
if ($Clean) {
    Write-Log "Cleaning previous builds..."
    if (Test-Path "$ProjectRoot\target") {
        Remove-Item "$ProjectRoot\target" -Recurse -Force
    }
    if (Test-Path "$ProjectRoot\guardian-ui\dist") {
        Remove-Item "$ProjectRoot\guardian-ui\dist" -Recurse -Force
    }
    if (Test-Path "$ProjectRoot\build") {
        Remove-Item "$ProjectRoot\build" -Recurse -Force
    }
    Write-Log "Clean completed" "SUCCESS"
}

# Create build directory structure
Write-Log "Creating build directory structure..."
$BuildDirs = @(
    "$ProjectRoot\build",
    "$ProjectRoot\build\executables",
    "$ProjectRoot\build\installers",
    "$ProjectRoot\build\logs",
    "$ProjectRoot\build\temp"
)

foreach ($dir in $BuildDirs) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
}
Write-Log "Build directories created" "SUCCESS"

# Build hostd backend
Write-Log "Building hostd backend..."
Set-Location "$ProjectRoot\hostd"
try {
    $CargoArgs = @("build", "--release")
    if ($Verbose) { $CargoArgs += "--verbose" }
    
    & cargo @CargoArgs
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo build failed with exit code $LASTEXITCODE"
    }
    
    # Copy executable to build directory
    Copy-Item "target\release\hostd.exe" "$ProjectRoot\build\executables\" -Force
    Write-Log "Hostd backend built successfully" "SUCCESS"
} catch {
    Write-Log "Failed to build hostd backend: $($_.Exception.Message)" "ERROR"
    exit 1
} finally {
    Set-Location $ProjectRoot
}

# Build GPU worker
Write-Log "Building GPU worker..."
Set-Location "$ProjectRoot\gpu-worker"
try {
    $CargoArgs = @("build", "--release")
    if ($Verbose) { $CargoArgs += "--verbose" }
    
    & cargo @CargoArgs
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo build failed with exit code $LASTEXITCODE"
    }
    
    # Copy executable to build directory
    Copy-Item "target\release\gpu-worker.exe" "$ProjectRoot\build\executables\" -Force
    Write-Log "GPU worker built successfully" "SUCCESS"
} catch {
    Write-Log "Failed to build GPU worker: $($_.Exception.Message)" "ERROR"
    exit 1
} finally {
    Set-Location $ProjectRoot
}

# Build Guardian UI frontend
Write-Log "Building Guardian UI frontend..."
Set-Location "$ProjectRoot\guardian-ui"
try {
    # Check if node_modules exists, if not install dependencies
    if (-not (Test-Path "node_modules")) {
        Write-Log "Installing frontend dependencies..."
        & npm install
        if ($LASTEXITCODE -ne 0) {
            throw "npm install failed with exit code $LASTEXITCODE"
        }
    } else {
        Write-Log "Using existing node_modules, skipping dependency installation" "INFO"
    }
    
    # Build frontend
    Write-Log "Building frontend..."
    & npm run build
    if ($LASTEXITCODE -ne 0) {
        throw "npm run build failed with exit code $LASTEXITCODE"
    }
    
    Write-Log "Frontend built successfully" "SUCCESS"
} catch {
    Write-Log "Failed to build frontend: $($_.Exception.Message)" "ERROR"
    exit 1
} finally {
    Set-Location $ProjectRoot
}

# Copy configuration files
Write-Log "Copying configuration files..."
$ConfigFiles = @(
    "configs\hostd.yaml",
    "configs\server.yaml", 
    "configs\rules.yaml",
    "configs\test.yaml"
)

foreach ($config in $ConfigFiles) {
    $SourcePath = "$ProjectRoot\$config"
    $DestPath = "$ProjectRoot\build\executables\$config"
    
    if (Test-Path $SourcePath) {
        $DestDir = Split-Path $DestPath -Parent
        if (-not (Test-Path $DestDir)) {
            New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
        }
        Copy-Item $SourcePath $DestPath -Force
        Write-Log "Copied $config" "INFO"
    } else {
        Write-Log "Warning: $config not found" "WARN"
    }
}

# Build Tauri application
Write-Log "Building Tauri application..."
Set-Location "$ProjectRoot\guardian-ui"
try {
    # Copy executables to Tauri directory
    Copy-Item "$ProjectRoot\build\executables\hostd.exe" "src-tauri\" -Force
    Copy-Item "$ProjectRoot\build\executables\gpu-worker.exe" "src-tauri\" -Force
    
    # Copy configs to Tauri directory
    $TauriConfigDir = "src-tauri\configs"
    if (-not (Test-Path $TauriConfigDir)) {
        New-Item -ItemType Directory -Path $TauriConfigDir -Force | Out-Null
    }
    Copy-Item "$ProjectRoot\configs\*" $TauriConfigDir -Force
    
    # Build Tauri app
    Write-Log "Building Tauri application..."
    & npm run tauri build
    if ($LASTEXITCODE -ne 0) {
        throw "Tauri build failed with exit code $LASTEXITCODE"
    }
    
    # Copy built installers to build directory
    $TauriDistDir = "src-tauri\target\release\bundle"
    if (Test-Path "$TauriDistDir\nsis") {
        Copy-Item "$TauriDistDir\nsis\*.exe" "$ProjectRoot\build\installers\" -Force
    }
    if (Test-Path "$TauriDistDir\msi") {
        Copy-Item "$TauriDistDir\msi\*.msi" "$ProjectRoot\build\installers\" -Force
    }
    
    Write-Log "Tauri application built successfully" "SUCCESS"
} catch {
    Write-Log "Failed to build Tauri application: $($_.Exception.Message)" "ERROR"
    exit 1
} finally {
    Set-Location $ProjectRoot
}

# Copy launcher scripts
Write-Log "Copying launcher scripts..."
$LauncherFiles = @(
    "launchers\start-guardian-with-backend.ps1",
    "launchers\start-guardian-production.bat"
)

foreach ($launcher in $LauncherFiles) {
    if (Test-Path "$ProjectRoot\$launcher") {
        Copy-Item "$ProjectRoot\$launcher" "$ProjectRoot\build\" -Force
        Write-Log "Copied $launcher" "INFO"
    }
}

# Create version information
Write-Log "Creating version information..."
$VersionInfo = @{
    version = "1.0.0"
    build_date = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
    git_commit = if (Test-Command "git") { 
        try { git rev-parse --short HEAD } catch { "unknown" }
    } else { "unknown" }
    build_environment = "production"
}

$VersionInfo | ConvertTo-Json | Out-File "$ProjectRoot\build\version.json" -Encoding UTF8
Write-Log "Version information created" "SUCCESS"

# Create build summary
Write-Log "Creating build summary..."
$BuildSummary = @"
Guardian Build Summary
=====================
Build Date: $($VersionInfo.build_date)
Version: $($VersionInfo.version)
Git Commit: $($VersionInfo.git_commit)

Built Components:
- hostd.exe (Backend)
- gpu-worker.exe (GPU Worker)
- Guardian UI (Frontend)
- Tauri Application (Desktop App)

Installers:
$(Get-ChildItem "$ProjectRoot\build\installers" -Name | ForEach-Object { "- $_" })

Launchers:
- start-guardian-with-backend.ps1
- start-guardian-production.bat

Build completed successfully!
"@

$BuildSummary | Out-File "$ProjectRoot\build\BUILD_SUMMARY.txt" -Encoding UTF8
Write-Log "Build summary created" "SUCCESS"

# Final success message
Write-Log "=== BUILD COMPLETED SUCCESSFULLY ===" "SUCCESS"
Write-Log "Build artifacts are in: $ProjectRoot\build" "INFO"
Write-Log "Installers are in: $ProjectRoot\build\installers" "INFO"
Write-Log "Executables are in: $ProjectRoot\build\executables" "INFO"

# Show build summary
Write-Host ""
Write-Host $BuildSummary -ForegroundColor Cyan

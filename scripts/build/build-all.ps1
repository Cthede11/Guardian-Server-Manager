# Master build script for Guardian project
# This script builds all components with comprehensive error handling and organizes everything in the build/ directory

param(
    [switch]$Clean,
    [switch]$SkipTests,
    [switch]$Verbose,
    [switch]$SkipCleanup
)

Write-Host "=== GUARDIAN MASTER BUILD SCRIPT ===" -ForegroundColor Green
Write-Host "Building Guardian Minecraft Server Manager with all fixes applied" -ForegroundColor Cyan

# Set error action preference
$ErrorActionPreference = "Stop"

# Get the project root directory
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Write-Host "Project root: $ProjectRoot" -ForegroundColor Blue

# Function to log with timestamp
function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $Color = switch ($Level) {
        "ERROR" { "Red" }
        "WARN" { "Yellow" }
        "SUCCESS" { "Green" }
        "BUILD" { "Cyan" }
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
Write-Log "Checking prerequisites..." "BUILD"
$Prerequisites = @("cargo", "npm", "node")
foreach ($cmd in $Prerequisites) {
    if (-not (Test-Command $cmd)) {
        Write-Log "Missing prerequisite: $cmd" "ERROR"
        exit 1
    }
}
Write-Log "All prerequisites found" "SUCCESS"

# Run cleanup first (unless skipped)
if (-not $SkipCleanup) {
    Write-Log "Running cleanup..." "BUILD"
    try {
        & "$ProjectRoot\scripts\cleanup.ps1"
        Write-Log "Cleanup completed successfully" "SUCCESS"
    } catch {
        Write-Log "Cleanup failed: $($_.Exception.Message)" "ERROR"
        Write-Log "Continuing without cleanup..." "WARN"
    }
}

# Build all components
Write-Log "Building all components..." "BUILD"

# 1. Build hostd backend
Write-Log "Building hostd backend..." "BUILD"
Set-Location "$ProjectRoot\hostd"
try {
    $CargoArgs = @("build", "--release")
    if ($Verbose) { $CargoArgs += "--verbose" }
    
    & cargo @CargoArgs
    if ($LASTEXITCODE -ne 0) {
        throw "Backend build failed with exit code $LASTEXITCODE"
    }
    
    Write-Log "Hostd backend built successfully" "SUCCESS"
} catch {
    Write-Log "Failed to build hostd backend: $($_.Exception.Message)" "ERROR"
    exit 1
} finally {
    Set-Location $ProjectRoot
}

# 2. Build GPU worker
Write-Log "Building GPU worker..." "BUILD"
Set-Location "$ProjectRoot\gpu-worker"
try {
    $CargoArgs = @("build", "--release")
    if ($Verbose) { $CargoArgs += "--verbose" }
    
    & cargo @CargoArgs
    if ($LASTEXITCODE -ne 0) {
        throw "GPU worker build failed with exit code $LASTEXITCODE"
    }
    
    Write-Log "GPU worker built successfully" "SUCCESS"
} catch {
    Write-Log "Failed to build GPU worker: $($_.Exception.Message)" "ERROR"
    exit 1
} finally {
    Set-Location $ProjectRoot
}

# 3. Build Guardian UI frontend
Write-Log "Building Guardian UI frontend..." "BUILD"
Set-Location "$ProjectRoot\guardian-ui"
try {
    # Install dependencies
    Write-Log "Installing frontend dependencies..." "BUILD"
    & npm ci
    if ($LASTEXITCODE -ne 0) {
        throw "npm ci failed with exit code $LASTEXITCODE"
    }
    
    # Build frontend
    Write-Log "Building frontend..." "BUILD"
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

# 4. Copy configuration files
Write-Log "Copying configuration files..." "BUILD"
$ConfigFiles = @(
    "configs/hostd.yaml",
    "configs/server.yaml", 
    "configs/rules.yaml",
    "configs/test.yaml"
)

foreach ($config in $ConfigFiles) {
    $SourcePath = "$ProjectRoot\$config"
    $DestPath = "$ProjectRoot\build/executables/$config"
    
    if (Test-Path $SourcePath) {
        $DestDir = Split-Path $DestPath -Parent
        if (-not (Test-Path $DestDir)) {
            New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
        }
        Copy-Item $SourcePath $DestPath -Force
        Write-Log "Copied $config" "BUILD"
    } else {
        Write-Log "Warning: $config not found" "WARN"
    }
}

# 5. Build Tauri application
Write-Log "Building Tauri application..." "BUILD"
Set-Location "$ProjectRoot\guardian-ui"
try {
    # Copy executables to Tauri directory
    if (Test-Path "$ProjectRoot\hostd/target/release/hostd.exe") {
        Copy-Item "$ProjectRoot\hostd/target/release/hostd.exe" "src-tauri/" -Force
        Write-Log "Copied hostd.exe to Tauri directory" "BUILD"
    } else {
        Write-Log "Warning: hostd.exe not found for Tauri build" "WARN"
    }
    
    if (Test-Path "$ProjectRoot\gpu-worker/target/release/gpu-worker.exe") {
        Copy-Item "$ProjectRoot\gpu-worker/target/release/gpu-worker.exe" "src-tauri/" -Force
        Write-Log "Copied gpu-worker.exe to Tauri directory" "BUILD"
    } else {
        Write-Log "Warning: gpu-worker.exe not found for Tauri build" "WARN"
    }
    
    # Copy configs to Tauri directory
    $TauriConfigDir = "src-tauri/configs"
    if (-not (Test-Path $TauriConfigDir)) {
        New-Item -ItemType Directory -Path $TauriConfigDir -Force | Out-Null
    }
    
    if (Test-Path "$ProjectRoot\configs") {
        Copy-Item "$ProjectRoot\configs/*" $TauriConfigDir -Force
        Write-Log "Copied config files to Tauri directory" "BUILD"
    } else {
        Write-Log "Warning: configs directory not found" "WARN"
    }
    
    # Build Tauri app
    Write-Log "Building Tauri application..." "BUILD"
    & npm run tauri build
    if ($LASTEXITCODE -ne 0) {
        throw "Tauri build failed with exit code $LASTEXITCODE"
    }
    
    Write-Log "Tauri application built successfully" "SUCCESS"
} catch {
    Write-Log "Failed to build Tauri application: $($_.Exception.Message)" "ERROR"
    exit 1
} finally {
    Set-Location $ProjectRoot
}

# 6. Organize build artifacts
Write-Log "Organizing build artifacts..." "BUILD"

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

# Copy backend executables
$BackendBinary = "$ProjectRoot\hostd/target/release/hostd.exe"
$GpuWorkerBinary = "$ProjectRoot\gpu-worker/target/release/gpu-worker.exe"

if (Test-Path $BackendBinary) {
    Copy-Item $BackendBinary "$ProjectRoot\build/executables/hostd.exe" -Force
    Write-Log "Copied hostd.exe to build/executables/" "SUCCESS"
} else {
    Write-Log "Warning: hostd.exe not found" "WARN"
}

if (Test-Path $GpuWorkerBinary) {
    Copy-Item $GpuWorkerBinary "$ProjectRoot\build/executables/gpu-worker.exe" -Force
    Write-Log "Copied gpu-worker.exe to build/executables/" "SUCCESS"
} else {
    Write-Log "Warning: gpu-worker.exe not found" "WARN"
}

# Copy Tauri installers
$TauriDistDir = "$ProjectRoot\guardian-ui/src-tauri/target/release/bundle"
if (Test-Path $TauriDistDir) {
    $BundleDirs = Get-ChildItem -Path $TauriDistDir -Directory
    foreach ($BundleDir in $BundleDirs) {
        $BundlePath = $BundleDir.FullName
        Write-Log "Copying Tauri bundle from: $BundlePath" "BUILD"
        
        # Copy all files from the bundle to installers directory
        $BundleName = $BundleDir.Name
        $TargetDir = "$ProjectRoot\build/installers/$BundleName"
        New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
        Copy-Item "$BundlePath/*" $TargetDir -Recurse -Force
        
        Write-Log "Copied Tauri bundle: $BundleName" "SUCCESS"
    }
} else {
    Write-Log "Warning: Tauri bundle directory not found" "WARN"
}

# 7. Copy launcher scripts
Write-Log "Copying launcher scripts..." "BUILD"
$LauncherFiles = @(
    "launchers/start-guardian-with-backend.ps1",
    "launchers/start-guardian-production.bat"
)

foreach ($launcher in $LauncherFiles) {
    if (Test-Path "$ProjectRoot\$launcher") {
        Copy-Item "$ProjectRoot\$launcher" "$ProjectRoot\build/" -Force
        Write-Log "Copied $launcher" "SUCCESS"
    }
}

# 8. Create version information
Write-Log "Creating version information..." "BUILD"
$VersionInfo = @{
    version = "1.0.0"
    build_date = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
    git_commit = if (Test-Command "git") { 
        try { git rev-parse --short HEAD } catch { "unknown" }
    } else { "unknown" }
    build_environment = "production"
    fixes_applied = @(
        "Console window elimination",
        "Backend connection race condition fixes", 
        "Process cleanup on app termination",
        "Professional startup experience",
        "Tauri webview security fixes",
        "HTTP request command implementation",
        "Enhanced debugging and logging"
    )
}

$VersionInfo | ConvertTo-Json | Out-File "$ProjectRoot\build/version.json" -Encoding UTF8
Write-Log "Version information created" "SUCCESS"

# 9. Create build summary
Write-Log "Creating build summary..." "BUILD"
$BuildSummary = @"
Guardian Build Summary
=====================
Build Date: $($VersionInfo.build_date)
Version: $($VersionInfo.version)
Git Commit: $($VersionInfo.git_commit)

Built Components:
- hostd.exe (Backend with CORS and process fixes)
- gpu-worker.exe (GPU Worker with process fixes)
- Guardian UI (Frontend with Tauri HTTP command integration)
- Tauri Application (Desktop App with HTTP command and cleanup handlers)

Fixes Applied:
- Console window elimination (CREATE_NO_WINDOW flag)
- Backend connection race condition prevention
- Process cleanup on app termination
- Professional startup experience
- Tauri webview security restrictions bypass
- HTTP request command implementation
- Enhanced debugging and error logging

Installers:
$(Get-ChildItem "$ProjectRoot\build/installers" -Recurse -Name | ForEach-Object { "- $_" })

Launchers:
- start-guardian-with-backend.ps1 (PowerShell with process cleanup)
- start-guardian-production.bat (Batch with process cleanup)

Build completed successfully!
"@

$BuildSummary | Out-File "$ProjectRoot\build/BUILD_SUMMARY.txt" -Encoding UTF8
Write-Log "Build summary created" "SUCCESS"

# Final success message
Write-Log "=== BUILD COMPLETED SUCCESSFULLY ===" "SUCCESS"
Write-Log "Build artifacts are in: $ProjectRoot\build" "BUILD"
Write-Log "Installers are in: $ProjectRoot\build/installers" "BUILD"
Write-Log "Executables are in: $ProjectRoot\build/executables" "BUILD"

# Show build summary
Write-Host ""
Write-Host $BuildSummary -ForegroundColor Cyan

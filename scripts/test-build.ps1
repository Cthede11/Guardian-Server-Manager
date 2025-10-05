# Test script to verify build-all.ps1 is ready for use
# This script performs a dry-run validation of the build process

param(
    [switch]$Verbose
)

Write-Host "=== GUARDIAN BUILD SCRIPT VALIDATION ===" -ForegroundColor Green
Write-Host "Testing build-all.ps1 script readiness..." -ForegroundColor Cyan

# Get the project root directory
$ProjectRoot = Split-Path -Parent $PSScriptRoot
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

# Test 1: Check if build script exists
Write-Log "Testing build script existence..." "BUILD"
$BuildScript = "$ProjectRoot\scripts\build\build-all.ps1"
if (Test-Path $BuildScript) {
    Write-Log "Build script found: $BuildScript" "SUCCESS"
} else {
    Write-Log "Build script not found: $BuildScript" "ERROR"
    exit 1
}

# Test 2: Check prerequisites
Write-Log "Testing prerequisites..." "BUILD"
$Prerequisites = @("cargo", "npm", "node")
$OptionalPrerequisites = @("java", "gradle")

$MissingRequired = @()
$MissingOptional = @()

foreach ($cmd in $Prerequisites) {
    if (-not (Test-Command $cmd)) {
        $MissingRequired += $cmd
    }
}

foreach ($cmd in $OptionalPrerequisites) {
    if (-not (Test-Command $cmd)) {
        $MissingOptional += $cmd
    }
}

if ($MissingRequired.Count -gt 0) {
    Write-Log "Missing required prerequisites: $($MissingRequired -join ', ')" "ERROR"
    Write-Log "Please install these tools before running the build script" "ERROR"
    exit 1
} else {
    Write-Log "All required prerequisites found" "SUCCESS"
}

if ($MissingOptional.Count -gt 0) {
    Write-Log "Missing optional prerequisites: $($MissingOptional -join ', ')" "WARN"
    Write-Log "Some features may be limited without these tools" "WARN"
}

# Test 3: Check project structure
Write-Log "Testing project structure..." "BUILD"
$RequiredDirs = @(
    "hostd",
    "gpu-worker", 
    "guardian-ui",
    "guardian-agent",
    "configs",
    "launchers"
)

$MissingDirs = @()
foreach ($dir in $RequiredDirs) {
    if (-not (Test-Path "$ProjectRoot\$dir")) {
        $MissingDirs += $dir
    }
}

if ($MissingDirs.Count -gt 0) {
    Write-Log "Missing required directories: $($MissingDirs -join ', ')" "ERROR"
    exit 1
} else {
    Write-Log "All required directories found" "SUCCESS"
}

# Test 4: Check key files
Write-Log "Testing key files..." "BUILD"
$RequiredFiles = @(
    "hostd\Cargo.toml",
    "gpu-worker\Cargo.toml",
    "guardian-ui\package.json",
    "guardian-ui\src-tauri\tauri.conf.json",
    "guardian-agent\build.gradle.kts"
)

$MissingFiles = @()
foreach ($file in $RequiredFiles) {
    if (-not (Test-Path "$ProjectRoot\$file")) {
        $MissingFiles += $file
    }
}

if ($MissingFiles.Count -gt 0) {
    Write-Log "Missing required files: $($MissingFiles -join ', ')" "ERROR"
    exit 1
} else {
    Write-Log "All required files found" "SUCCESS"
}

# Test 5: Check PowerShell execution policy
Write-Log "Testing PowerShell execution policy..." "BUILD"
$ExecutionPolicy = Get-ExecutionPolicy
if ($ExecutionPolicy -eq "Restricted") {
    Write-Log "PowerShell execution policy is restricted. Scripts may not run." "ERROR"
    Write-Log "Please run: Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser" "ERROR"
    exit 1
} else {
    Write-Log "PowerShell execution policy allows script execution: $ExecutionPolicy" "SUCCESS"
}

# Test 6: Check build directories
Write-Log "Testing build directory structure..." "BUILD"
$BuildDirs = @(
    "build/executables",
    "build/installers", 
    "build/temp",
    "build/logs"
)

foreach ($Dir in $BuildDirs) {
    $FullPath = Join-Path $ProjectRoot $Dir
    if (-not (Test-Path $FullPath)) {
        New-Item -ItemType Directory -Path $FullPath -Force | Out-Null
        Write-Log "Created build directory: $Dir" "BUILD"
    }
}

Write-Log "Build directory structure ready" "SUCCESS"

# Final validation
Write-Log "=== BUILD SCRIPT VALIDATION COMPLETE ===" "SUCCESS"
Write-Log "The build-all.ps1 script is ready for use!" "SUCCESS"
Write-Log "Run: .\scripts\build\build-all.ps1" "BUILD"
Write-Log "Or with options: .\scripts\build\build-all.ps1 -Clean -Verbose" "BUILD"

Write-Host ""
Write-Host "Build script validation completed successfully!" -ForegroundColor Green
Write-Host "You can now run the build script with confidence." -ForegroundColor Cyan

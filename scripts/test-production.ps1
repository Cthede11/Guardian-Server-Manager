# Guardian Production Testing Script
# This script tests the built application for console windows, process cleanup, and functionality

param(
    [switch]$Verbose,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

Write-Host "=== GUARDIAN PRODUCTION TESTING ===" -ForegroundColor Green

# Function to log with timestamp
function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $Color = switch ($Level) {
        "ERROR" { "Red" }
        "WARN" { "Yellow" }
        "SUCCESS" { "Green" }
        "TEST" { "Cyan" }
        default { "White" }
    }
    Write-Host "[$Timestamp] [$Level] $Message" -ForegroundColor $Color
}

# Function to test if process is running
function Test-ProcessRunning {
    param([string]$ProcessName)
    $Processes = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue
    return $Processes.Count -gt 0
}

# Function to wait for process to start
function Wait-ForProcess {
    param([string]$ProcessName, [int]$TimeoutSeconds = 10)
    $StartTime = Get-Date
    while ((Get-Date) - $StartTime -lt [TimeSpan]::FromSeconds($TimeoutSeconds)) {
        if (Test-ProcessRunning $ProcessName) {
            return $true
        }
        Start-Sleep -Milliseconds 500
    }
    return $false
}

# Function to wait for process to stop
function Wait-ForProcessToStop {
    param([string]$ProcessName, [int]$TimeoutSeconds = 10)
    $StartTime = Get-Date
    while ((Get-Date) - $StartTime -lt [TimeSpan]::FromSeconds($TimeoutSeconds)) {
        if (-not (Test-ProcessRunning $ProcessName)) {
            return $true
        }
        Start-Sleep -Milliseconds 500
    }
    return $false
}

# Function to test HTTP endpoint
function Test-HttpEndpoint {
    param([string]$Url, [int]$TimeoutSeconds = 5)
    try {
        $Response = Invoke-WebRequest -Uri $Url -Method GET -TimeoutSec $TimeoutSeconds -ErrorAction Stop
        return $Response.StatusCode -eq 200
    } catch {
        return $false
    }
}

# Test 1: Check if build exists
Write-Log "Test 1: Checking build artifacts..." "TEST"
$RequiredFiles = @(
    "$ProjectRoot\build\executables\hostd.exe",
    "$ProjectRoot\build\executables\gpu-worker.exe",
    "$ProjectRoot\build\installers\Guardian_1.0.0_x64-setup.exe"
)

$AllFilesExist = $true
foreach ($file in $RequiredFiles) {
    if (Test-Path $file) {
        Write-Log "✓ Found: $(Split-Path $file -Leaf)" "SUCCESS"
    } else {
        Write-Log "✗ Missing: $(Split-Path $file -Leaf)" "ERROR"
        $AllFilesExist = $false
    }
}

if (-not $AllFilesExist) {
    Write-Log "Build artifacts missing. Run build script first." "ERROR"
    exit 1
}

# Test 2: Test hostd executable (no console window)
Write-Log "Test 2: Testing hostd executable (no console window)..." "TEST"
$HostdPath = "$ProjectRoot\build\executables\hostd.exe"

# Start hostd in background
Write-Log "Starting hostd process..." "INFO"
$HostdProcess = Start-Process -FilePath $HostdPath -ArgumentList "--help" -PassThru -WindowStyle Hidden

# Wait a moment for process to start
Start-Sleep -Seconds 2

# Check if process started
if ($HostdProcess.HasExited) {
    Write-Log "✗ hostd process exited immediately" "ERROR"
    Write-Log "Exit code: $($HostdProcess.ExitCode)" "ERROR"
} else {
    Write-Log "✓ hostd process started successfully" "SUCCESS"
    Write-Log "PID: $($HostdProcess.Id)" "INFO"
    
    # Check for console windows
    $ConsoleWindows = Get-Process | Where-Object { $_.MainWindowTitle -like "*hostd*" -or $_.ProcessName -like "*cmd*" -or $_.ProcessName -like "*powershell*" } | Where-Object { $_.Id -ne $PID }
    if ($ConsoleWindows.Count -eq 0) {
        Write-Log "✓ No console windows detected" "SUCCESS"
    } else {
        Write-Log "✗ Console windows detected:" "ERROR"
        foreach ($window in $ConsoleWindows) {
            Write-Log "  - $($window.ProcessName) (PID: $($window.Id))" "ERROR"
        }
    }
    
    # Terminate hostd
    Write-Log "Terminating hostd process..." "INFO"
    $HostdProcess.Kill()
    $HostdProcess.WaitForExit(5000)
    Write-Log "✓ hostd process terminated" "SUCCESS"
}

# Test 3: Test GPU worker executable (no console window)
Write-Log "Test 3: Testing GPU worker executable (no console window)..." "TEST"
$GpuWorkerPath = "$ProjectRoot\build\executables\gpu-worker.exe"

if (Test-Path $GpuWorkerPath) {
    # Start GPU worker in background
    Write-Log "Starting GPU worker process..." "INFO"
    $GpuWorkerProcess = Start-Process -FilePath $GpuWorkerPath -PassThru -WindowStyle Hidden
    
    # Wait a moment for process to start
    Start-Sleep -Seconds 2
    
    # Check if process started
    if ($GpuWorkerProcess.HasExited) {
        Write-Log "✗ GPU worker process exited immediately" "ERROR"
        Write-Log "Exit code: $($GpuWorkerProcess.ExitCode)" "ERROR"
    } else {
        Write-Log "✓ GPU worker process started successfully" "SUCCESS"
        Write-Log "PID: $($GpuWorkerProcess.Id)" "INFO"
        
        # Check for console windows
        $ConsoleWindows = Get-Process | Where-Object { $_.MainWindowTitle -like "*gpu-worker*" -or $_.ProcessName -like "*cmd*" -or $_.ProcessName -like "*powershell*" } | Where-Object { $_.Id -ne $PID }
        if ($ConsoleWindows.Count -eq 0) {
            Write-Log "✓ No console windows detected" "SUCCESS"
        } else {
            Write-Log "✗ Console windows detected:" "ERROR"
            foreach ($window in $ConsoleWindows) {
                Write-Log "  - $($window.ProcessName) (PID: $($window.Id))" "ERROR"
            }
        }
        
        # Terminate GPU worker
        Write-Log "Terminating GPU worker process..." "INFO"
        $GpuWorkerProcess.Kill()
        $GpuWorkerProcess.WaitForExit(5000)
        Write-Log "✓ GPU worker process terminated" "SUCCESS"
    }
} else {
    Write-Log "✗ GPU worker executable not found" "ERROR"
}

# Test 4: Test launcher scripts
Write-Log "Test 4: Testing launcher scripts..." "TEST"

# Test PowerShell launcher
$PsLauncherPath = "$ProjectRoot\build\start-guardian-with-backend.ps1"
if (Test-Path $PsLauncherPath) {
    Write-Log "✓ PowerShell launcher found" "SUCCESS"
} else {
    Write-Log "✗ PowerShell launcher not found" "ERROR"
}

# Test Batch launcher
$BatchLauncherPath = "$ProjectRoot\build\start-guardian-production.bat"
if (Test-Path $BatchLauncherPath) {
    Write-Log "✓ Batch launcher found" "SUCCESS"
} else {
    Write-Log "✗ Batch launcher not found" "ERROR"
}

# Test 5: Test installer
Write-Log "Test 5: Testing installer..." "TEST"
$InstallerPath = "$ProjectRoot\build\installers\Guardian_1.0.0_x64-setup.exe"
if (Test-Path $InstallerPath) {
    Write-Log "✓ Installer found" "SUCCESS"
    $InstallerSize = (Get-Item $InstallerPath).Length / 1MB
    Write-Log "Installer size: $([math]::Round($InstallerSize, 2)) MB" "INFO"
} else {
    Write-Log "✗ Installer not found" "ERROR"
}

# Test 6: Test process cleanup
Write-Log "Test 6: Testing process cleanup..." "TEST"

# Start multiple processes
Write-Log "Starting test processes..." "INFO"
$TestProcesses = @()

# Start hostd
$HostdProcess = Start-Process -FilePath $HostdPath -ArgumentList "--help" -PassThru -WindowStyle Hidden
$TestProcesses += $HostdProcess

# Start GPU worker if available
if (Test-Path $GpuWorkerPath) {
    $GpuWorkerProcess = Start-Process -FilePath $GpuWorkerPath -PassThru -WindowStyle Hidden
    $TestProcesses += $GpuWorkerProcess
}

# Wait for processes to start
Start-Sleep -Seconds 2

# Count running processes
$RunningProcesses = $TestProcesses | Where-Object { -not $_.HasExited }
Write-Log "Started $($RunningProcesses.Count) test processes" "INFO"

# Cleanup processes
Write-Log "Cleaning up test processes..." "INFO"
foreach ($process in $TestProcesses) {
    if (-not $process.HasExited) {
        $process.Kill()
        $process.WaitForExit(5000)
    }
}

# Wait for cleanup
Start-Sleep -Seconds 2

# Check if processes are cleaned up
$RemainingProcesses = $TestProcesses | Where-Object { -not $_.HasExited }
if ($RemainingProcesses.Count -eq 0) {
    Write-Log "✓ All test processes cleaned up successfully" "SUCCESS"
} else {
    Write-Log "✗ $($RemainingProcesses.Count) processes still running" "ERROR"
    foreach ($process in $RemainingProcesses) {
        Write-Log "  - PID: $($process.Id)" "ERROR"
    }
}

# Test Summary
Write-Log "=== TEST SUMMARY ===" "TEST"
Write-Log "All tests completed. Check results above." "INFO"

# Final cleanup
Write-Log "Performing final cleanup..." "INFO"
Get-Process | Where-Object { $_.ProcessName -like "*hostd*" -or $_.ProcessName -like "*gpu-worker*" } | Where-Object { $_.Id -ne $PID } | ForEach-Object {
    Write-Log "Cleaning up remaining process: $($_.ProcessName) (PID: $($_.Id))" "WARN"
    try {
        $_.Kill()
    } catch {
        Write-Log "Failed to kill process $($_.Id): $($_.Exception.Message)" "WARN"
    }
}

Write-Log "=== TESTING COMPLETED ===" "SUCCESS"

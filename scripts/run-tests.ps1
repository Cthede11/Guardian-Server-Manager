# Guardian Test Runner Script
# This script runs all tests for the Guardian project

param(
    [switch]$Unit = $false,
    [switch]$Integration = $false,
    [switch]$E2E = $false,
    [switch]$All = $true,
    [switch]$Verbose = $false,
    [switch]$Coverage = $false,
    [string]$Filter = "",
    [switch]$Clean = $false
)

# Set error action preference
$ErrorActionPreference = "Stop"

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Blue"
$Cyan = "Cyan"

function Write-ColorOutput {
    param([string]$Message, [string]$Color = "White")
    Write-Host $Message -ForegroundColor $Color
}

function Write-TestHeader {
    param([string]$Title)
    Write-ColorOutput "`n" + "="*60 $Cyan
    Write-ColorOutput " $Title" $Cyan
    Write-ColorOutput "="*60 $Cyan
}

function Write-TestResult {
    param([string]$TestName, [bool]$Passed, [string]$Details = "")
    if ($Passed) {
        Write-ColorOutput "✅ $TestName" $Green
    } else {
        Write-ColorOutput "❌ $TestName" $Red
        if ($Details) {
            Write-ColorOutput "   $Details" $Red
        }
    }
}

function Test-BackendUnitTests {
    Write-TestHeader "Backend Unit Tests"
    
    try {
        Push-Location "hostd"
        
        $testArgs = @("test")
        if ($Verbose) { $testArgs += "--verbose" }
        if ($Filter) { $testArgs += "--", $Filter }
        if ($Coverage) { $testArgs += "--", "--nocapture" }
        
        $result = & cargo $testArgs 2>&1
        $exitCode = $LASTEXITCODE
        
        if ($exitCode -eq 0) {
            Write-TestResult "Backend Unit Tests" $true
            if ($Verbose) {
                Write-ColorOutput $result $Yellow
            }
        } else {
            Write-TestResult "Backend Unit Tests" $false $result
        }
        
        Pop-Location
        return $exitCode -eq 0
    }
    catch {
        Write-TestResult "Backend Unit Tests" $false $_.Exception.Message
        Pop-Location
        return $false
    }
}

function Test-GPUWorkerUnitTests {
    Write-TestHeader "GPU Worker Unit Tests"
    
    try {
        Push-Location "gpu-worker"
        
        $testArgs = @("test")
        if ($Verbose) { $testArgs += "--verbose" }
        if ($Filter) { $testArgs += "--", $Filter }
        
        $result = & cargo $testArgs 2>&1
        $exitCode = $LASTEXITCODE
        
        if ($exitCode -eq 0) {
            Write-TestResult "GPU Worker Unit Tests" $true
            if ($Verbose) {
                Write-ColorOutput $result $Yellow
            }
        } else {
            Write-TestResult "GPU Worker Unit Tests" $false $result
        }
        
        Pop-Location
        return $exitCode -eq 0
    }
    catch {
        Write-TestResult "GPU Worker Unit Tests" $false $_.Exception.Message
        Pop-Location
        return $false
    }
}

function Test-FrontendUnitTests {
    Write-TestHeader "Frontend Unit Tests"
    
    try {
        Push-Location "guardian-ui"
        
        $testArgs = @("test")
        if ($Verbose) { $testArgs += "--verbose" }
        if ($Filter) { $testArgs += "--", $Filter }
        
        $result = & npm $testArgs 2>&1
        $exitCode = $LASTEXITCODE
        
        if ($exitCode -eq 0) {
            Write-TestResult "Frontend Unit Tests" $true
            if ($Verbose) {
                Write-ColorOutput $result $Yellow
            }
        } else {
            Write-TestResult "Frontend Unit Tests" $false $result
        }
        
        Pop-Location
        return $exitCode -eq 0
    }
    catch {
        Write-TestResult "Frontend Unit Tests" $false $_.Exception.Message
        Pop-Location
        return $false
    }
}

function Test-IntegrationTests {
    Write-TestHeader "Integration Tests"
    
    try {
        Push-Location "hostd"
        
        $testArgs = @("test", "--test", "integration_tests")
        if ($Verbose) { $testArgs += "--verbose" }
        if ($Filter) { $testArgs += "--", $Filter }
        
        $result = & cargo $testArgs 2>&1
        $exitCode = $LASTEXITCODE
        
        if ($exitCode -eq 0) {
            Write-TestResult "Integration Tests" $true
            if ($Verbose) {
                Write-ColorOutput $result $Yellow
            }
        } else {
            Write-TestResult "Integration Tests" $false $result
        }
        
        Pop-Location
        return $exitCode -eq 0
    }
    catch {
        Write-TestResult "Integration Tests" $false $_.Exception.Message
        Pop-Location
        return $false
    }
}

function Test-E2ETests {
    Write-TestHeader "End-to-End Tests"
    
    try {
        # Check if Guardian backend is running
        $health = try {
            Invoke-RestMethod -Uri "http://localhost:8080/api/health" -Method GET -TimeoutSec 5
        } catch {
            $null
        }
        
        if (-not $health) {
            Write-ColorOutput "Starting Guardian backend for E2E tests..." $Yellow
            
            # Start backend in background
            $backendJob = Start-Job -ScriptBlock {
                Set-Location "hostd"
                & cargo run --release
            }
            
            # Wait for backend to start
            $maxWait = 30
            $waited = 0
            while ($waited -lt $maxWait) {
                Start-Sleep -Seconds 2
                $waited += 2
                
                $health = try {
                    Invoke-RestMethod -Uri "http://localhost:8080/api/health" -Method GET -TimeoutSec 2
                } catch {
                    $null
                }
                
                if ($health) {
                    Write-ColorOutput "Backend started successfully" $Green
                    break
                }
            }
            
            if (-not $health) {
                Write-TestResult "E2E Tests" $false "Failed to start backend"
                Stop-Job $backendJob -Force
                Remove-Job $backendJob
                return $false
            }
        }
        
        # Run E2E test script
        $e2eResult = & ".\scripts\test-zero-downtime.ps1" -Verbose:$Verbose
        $exitCode = $LASTEXITCODE
        
        if ($exitCode -eq 0) {
            Write-TestResult "E2E Tests" $true
        } else {
            Write-TestResult "E2E Tests" $false "Zero-downtime test failed"
        }
        
        # Clean up backend if we started it
        if ($backendJob) {
            Stop-Job $backendJob -Force
            Remove-Job $backendJob
        }
        
        return $exitCode -eq 0
    }
    catch {
        Write-TestResult "E2E Tests" $false $_.Exception.Message
        return $false
    }
}

function Test-BuildTests {
    Write-TestHeader "Build Tests"
    
    $buildResults = @()
    
    # Test backend build
    try {
        Push-Location "hostd"
        $result = & cargo build --release 2>&1
        $exitCode = $LASTEXITCODE
        Pop-Location
        
        if ($exitCode -eq 0) {
            Write-TestResult "Backend Build" $true
            $buildResults += $true
        } else {
            Write-TestResult "Backend Build" $false $result
            $buildResults += $false
        }
    }
    catch {
        Write-TestResult "Backend Build" $false $_.Exception.Message
        $buildResults += $false
        Pop-Location
    }
    
    # Test GPU worker build
    try {
        Push-Location "gpu-worker"
        $result = & cargo build --release 2>&1
        $exitCode = $LASTEXITCODE
        Pop-Location
        
        if ($exitCode -eq 0) {
            Write-TestResult "GPU Worker Build" $true
            $buildResults += $true
        } else {
            Write-TestResult "GPU Worker Build" $false $result
            $buildResults += $false
        }
    }
    catch {
        Write-TestResult "GPU Worker Build" $false $_.Exception.Message
        $buildResults += $false
        Pop-Location
    }
    
    # Test frontend build
    try {
        Push-Location "guardian-ui"
        $result = & npm run build 2>&1
        $exitCode = $LASTEXITCODE
        Pop-Location
        
        if ($exitCode -eq 0) {
            Write-TestResult "Frontend Build" $true
            $buildResults += $true
        } else {
            Write-TestResult "Frontend Build" $false $result
            $buildResults += $false
        }
    }
    catch {
        Write-TestResult "Frontend Build" $false $_.Exception.Message
        $buildResults += $false
        Pop-Location
    }
    
    # Test Tauri build
    try {
        Push-Location "guardian-ui"
        $result = & npm run tauri:build 2>&1
        $exitCode = $LASTEXITCODE
        Pop-Location
        
        if ($exitCode -eq 0) {
            Write-TestResult "Tauri Build" $true
            $buildResults += $true
        } else {
            Write-TestResult "Tauri Build" $false $result
            $buildResults += $false
        }
    }
    catch {
        Write-TestResult "Tauri Build" $false $_.Exception.Message
        $buildResults += $false
        Pop-Location
    }
    
    return ($buildResults -contains $false) -eq $false
}

function Test-CodeQuality {
    Write-TestHeader "Code Quality Tests"
    
    $qualityResults = @()
    
    # Test Rust formatting
    try {
        Push-Location "hostd"
        $result = & cargo fmt --check 2>&1
        $exitCode = $LASTEXITCODE
        Pop-Location
        
        if ($exitCode -eq 0) {
            Write-TestResult "Rust Formatting" $true
            $qualityResults += $true
        } else {
            Write-TestResult "Rust Formatting" $false "Run 'cargo fmt' to fix formatting issues"
            $qualityResults += $false
        }
    }
    catch {
        Write-TestResult "Rust Formatting" $false $_.Exception.Message
        $qualityResults += $false
        Pop-Location
    }
    
    # Test Rust clippy
    try {
        Push-Location "hostd"
        $result = & cargo clippy -- -D warnings 2>&1
        $exitCode = $LASTEXITCODE
        Pop-Location
        
        if ($exitCode -eq 0) {
            Write-TestResult "Rust Clippy" $true
            $qualityResults += $true
        } else {
            Write-TestResult "Rust Clippy" $false $result
            $qualityResults += $false
        }
    }
    catch {
        Write-TestResult "Rust Clippy" $false $_.Exception.Message
        $qualityResults += $false
        Pop-Location
    }
    
    # Test TypeScript linting
    try {
        Push-Location "guardian-ui"
        $result = & npm run lint 2>&1
        $exitCode = $LASTEXITCODE
        Pop-Location
        
        if ($exitCode -eq 0) {
            Write-TestResult "TypeScript Linting" $true
            $qualityResults += $true
        } else {
            Write-TestResult "TypeScript Linting" $false $result
            $qualityResults += $false
        }
    }
    catch {
        Write-TestResult "TypeScript Linting" $false $_.Exception.Message
        $qualityResults += $false
        Pop-Location
    }
    
    return ($qualityResults -contains $false) -eq $false
}

function Test-PerformanceTests {
    Write-TestHeader "Performance Tests"
    
    try {
        # Test database performance
        Push-Location "hostd"
        $result = & cargo test --test performance_tests 2>&1
        $exitCode = $LASTEXITCODE
        Pop-Location
        
        if ($exitCode -eq 0) {
            Write-TestResult "Database Performance" $true
        } else {
            Write-TestResult "Database Performance" $false $result
        }
        
        return $exitCode -eq 0
    }
    catch {
        Write-TestResult "Performance Tests" $false $_.Exception.Message
        Pop-Location
        return $false
    }
}

function Test-Cleanup {
    Write-TestHeader "Cleanup"
    
    try {
        # Clean build artifacts
        if (Test-Path "hostd\target") {
            Remove-Item "hostd\target" -Recurse -Force
            Write-ColorOutput "Cleaned hostd target directory" $Green
        }
        
        if (Test-Path "gpu-worker\target") {
            Remove-Item "gpu-worker\target" -Recurse -Force
            Write-ColorOutput "Cleaned gpu-worker target directory" $Green
        }
        
        if (Test-Path "guardian-ui\dist") {
            Remove-Item "guardian-ui\dist" -Recurse -Force
            Write-ColorOutput "Cleaned frontend dist directory" $Green
        }
        
        if (Test-Path "guardian-ui\src-tauri\target") {
            Remove-Item "guardian-ui\src-tauri\target" -Recurse -Force
            Write-ColorOutput "Cleaned Tauri target directory" $Green
        }
        
        if (Test-Path "build") {
            Remove-Item "build" -Recurse -Force
            Write-ColorOutput "Cleaned build directory" $Green
        }
        
        Write-TestResult "Cleanup" $true
        return $true
    }
    catch {
        Write-TestResult "Cleanup" $false $_.Exception.Message
        return $false
    }
}

# Main execution
Write-ColorOutput "`n🧪 Guardian Test Runner" $Blue
Write-ColorOutput "=====================" $Blue

$testResults = @()
$startTime = Get-Date

# Cleanup if requested
if ($Clean) {
    Test-Cleanup | Out-Null
}

# Run tests based on parameters
if ($All -or $Unit) {
    $testResults += Test-BackendUnitTests
    $testResults += Test-GPUWorkerUnitTests
    $testResults += Test-FrontendUnitTests
}

if ($All -or $Integration) {
    $testResults += Test-IntegrationTests
}

if ($All -or $E2E) {
    $testResults += Test-E2ETests
}

if ($All) {
    $testResults += Test-BuildTests
    $testResults += Test-CodeQuality
    $testResults += Test-PerformanceTests
}

$endTime = Get-Date
$duration = $endTime - $startTime

# Summary
Write-TestHeader "Test Summary"
$passed = ($testResults | Where-Object { $_ -eq $true }).Count
$total = $testResults.Count
$failed = $total - $passed

Write-ColorOutput "Total Tests: $total" $Cyan
Write-ColorOutput "Passed: $passed" $Green
Write-ColorOutput "Failed: $failed" $(if ($failed -gt 0) { $Red } else { $Green })
Write-ColorOutput "Duration: $($duration.ToString('hh\:mm\:ss'))" $Cyan

if ($failed -gt 0) {
    Write-ColorOutput "`n❌ Some tests failed!" $Red
    exit 1
} else {
    Write-ColorOutput "`n✅ All tests passed!" $Green
    exit 0
}

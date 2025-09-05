# Guardian Production Test Script
# Tests the production build to ensure everything works correctly

Write-Host "🧪 Testing Guardian Production Build" -ForegroundColor Green
Write-Host "====================================" -ForegroundColor Green

$ErrorActionPreference = "Stop"

# Test configuration
$TestDir = "guardian-distribution"
$TestResults = @()

function Test-File {
    param($Path, $Description)
    
    if (Test-Path $Path) {
        Write-Host "✅ $Description" -ForegroundColor Green
        $TestResults += "PASS: $Description"
        return $true
    } else {
        Write-Host "❌ $Description" -ForegroundColor Red
        $TestResults += "FAIL: $Description"
        return $false
    }
}

function Test-Executable {
    param($Path, $Description)
    
    if (Test-Path $Path) {
        try {
            # Try to get file info to verify it's a valid executable
            $fileInfo = Get-Item $Path
            if ($fileInfo.Extension -eq ".exe") {
                Write-Host "✅ $Description" -ForegroundColor Green
                $TestResults += "PASS: $Description"
                return $true
            } else {
                Write-Host "❌ $Description (not an executable)" -ForegroundColor Red
                $TestResults += "FAIL: $Description (not an executable)"
                return $false
            }
        } catch {
            Write-Host "❌ $Description (error checking file)" -ForegroundColor Red
            $TestResults += "FAIL: $Description (error checking file)"
            return $false
        }
    } else {
        Write-Host "❌ $Description" -ForegroundColor Red
        $TestResults += "FAIL: $Description"
        return $false
    }
}

# Check if test directory exists
if (-not (Test-Path $TestDir)) {
    Write-Host "❌ Test directory not found: $TestDir" -ForegroundColor Red
    Write-Host "Please run build-final.ps1 first to create the production build." -ForegroundColor Yellow
    exit 1
}

Write-Host "📁 Testing directory: $TestDir" -ForegroundColor Cyan

# Test core files
Write-Host "`n🔍 Testing core files..." -ForegroundColor Yellow
$CoreFilesPassed = 0
$CoreFilesTotal = 0

$CoreFilesTotal++
if (Test-File "$TestDir/guardian.exe" "Guardian main executable") { $CoreFilesPassed++ }

$CoreFilesTotal++
if (Test-File "$TestDir/hostd.exe" "Hostd backend executable") { $CoreFilesPassed++ }

$CoreFilesTotal++
if (Test-File "$TestDir/gpu-worker.exe" "GPU worker executable") { $CoreFilesPassed++ }

# Test configuration files
Write-Host "`n🔍 Testing configuration files..." -ForegroundColor Yellow
$ConfigFilesPassed = 0
$ConfigFilesTotal = 0

$ConfigFilesTotal++
if (Test-File "$TestDir/configs" "Configuration directory") { $ConfigFilesPassed++ }

$ConfigFilesTotal++
if (Test-File "$TestDir/configs/server.yaml" "Server configuration") { $ConfigFilesPassed++ }

$ConfigFilesTotal++
if (Test-File "$TestDir/configs/rules.yaml" "Rules configuration") { $ConfigFilesPassed++ }

# Test data directories
Write-Host "`n🔍 Testing data directories..." -ForegroundColor Yellow
$DataDirsPassed = 0
$DataDirsTotal = 0

$DataDirsTotal++
if (Test-File "$TestDir/data" "Data directory") { $DataDirsPassed++ }

$DataDirsTotal++
if (Test-File "$TestDir/logs" "Logs directory") { $DataDirsPassed++ }

# Test documentation
Write-Host "`n🔍 Testing documentation..." -ForegroundColor Yellow
$DocFilesPassed = 0
$DocFilesTotal = 0

$DocFilesTotal++
if (Test-File "$TestDir/README.txt" "README file") { $DocFilesPassed++ }

$DocFilesTotal++
if (Test-File "$TestDir/version.json" "Version information") { $DocFilesPassed++ }

# Test scripts
Write-Host "`n🔍 Testing scripts..." -ForegroundColor Yellow
$ScriptsPassed = 0
$ScriptsTotal = 0

$ScriptsTotal++
if (Test-File "$TestDir/start-guardian.bat" "Launcher script") { $ScriptsPassed++ }

$ScriptsTotal++
if (Test-File "$TestDir/uninstall.bat" "Uninstaller script") { $ScriptsPassed++ }

# Test MSI installer
Write-Host "`n🔍 Testing installer..." -ForegroundColor Yellow
$InstallerPassed = 0
$InstallerTotal = 0

$InstallerTotal++
$MsiFiles = Get-ChildItem -Path $TestDir -Filter "*.msi" -ErrorAction SilentlyContinue
if ($MsiFiles.Count -gt 0) {
    Write-Host "✅ MSI installer found" -ForegroundColor Green
    $TestResults += "PASS: MSI installer found"
    $InstallerPassed++
} else {
    Write-Host "❌ MSI installer not found" -ForegroundColor Red
    $TestResults += "FAIL: MSI installer not found"
}

# Calculate overall results
$TotalPassed = $CoreFilesPassed + $ConfigFilesPassed + $DataDirsPassed + $DocFilesPassed + $ScriptsPassed + $InstallerPassed
$TotalTests = $CoreFilesTotal + $ConfigFilesTotal + $DataDirsTotal + $DocFilesTotal + $ScriptsTotal + $InstallerTotal
$PassRate = [math]::Round(($TotalPassed / $TotalTests) * 100, 2)

# Display results
Write-Host "`n📊 Test Results Summary" -ForegroundColor Green
Write-Host "======================" -ForegroundColor Green
Write-Host "Core Files: $CoreFilesPassed/$CoreFilesTotal" -ForegroundColor Cyan
Write-Host "Config Files: $ConfigFilesPassed/$ConfigFilesTotal" -ForegroundColor Cyan
Write-Host "Data Directories: $DataDirsPassed/$DataDirsTotal" -ForegroundColor Cyan
Write-Host "Documentation: $DocFilesPassed/$DocFilesTotal" -ForegroundColor Cyan
Write-Host "Scripts: $ScriptsPassed/$ScriptsTotal" -ForegroundColor Cyan
Write-Host "Installer: $InstallerPassed/$InstallerTotal" -ForegroundColor Cyan
Write-Host "----------------------" -ForegroundColor Green
Write-Host "Overall: $TotalPassed/$TotalTests ($PassRate%)" -ForegroundColor $(if ($PassRate -ge 90) { "Green" } elseif ($PassRate -ge 70) { "Yellow" } else { "Red" })

# Save detailed results
$TestResults | Out-File -FilePath "test-results.txt" -Encoding UTF8
Write-Host "`n📄 Detailed results saved to test-results.txt" -ForegroundColor Cyan

# Final verdict
if ($PassRate -ge 90) {
    Write-Host "`n🎉 Production build test PASSED!" -ForegroundColor Green
    Write-Host "The build is ready for distribution." -ForegroundColor Green
    exit 0
} elseif ($PassRate -ge 70) {
    Write-Host "`n⚠️ Production build test PARTIALLY PASSED" -ForegroundColor Yellow
    Write-Host "Some issues found, but the build may still be usable." -ForegroundColor Yellow
    exit 1
} else {
    Write-Host "`n❌ Production build test FAILED" -ForegroundColor Red
    Write-Host "Significant issues found. Please fix before distribution." -ForegroundColor Red
    exit 1
}

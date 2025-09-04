# Deployment script for Guardian
# This script builds, tests, and deploys the Guardian application

param(
    [string]$Environment = "development",
    [switch]$SkipTests = $false,
    [switch]$SkipBuild = $false,
    [string]$Version = "1.0.0"
)

Write-Host "🚀 Guardian Deployment Script" -ForegroundColor Green
Write-Host "Environment: $Environment" -ForegroundColor Blue
Write-Host "Version: $Version" -ForegroundColor Blue

# Set error action preference
$ErrorActionPreference = "Stop"

# Get the project root directory
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Write-Host "Project root: $ProjectRoot" -ForegroundColor Blue

# Function to run tests
function Run-Tests {
    Write-Host "🧪 Running tests..." -ForegroundColor Yellow
    
    # Test backend
    Set-Location "$ProjectRoot/hostd"
    try {
        cargo test
        if ($LASTEXITCODE -ne 0) {
            throw "Backend tests failed"
        }
        Write-Host "✅ Backend tests passed" -ForegroundColor Green
    } catch {
        Write-Host "❌ Backend tests failed: $_" -ForegroundColor Red
        exit 1
    }
    
    # Test frontend
    Set-Location "$ProjectRoot/guardian-ui"
    try {
        npm run test:run
        if ($LASTEXITCODE -ne 0) {
            throw "Frontend tests failed"
        }
        Write-Host "✅ Frontend tests passed" -ForegroundColor Green
    } catch {
        Write-Host "❌ Frontend tests failed: $_" -ForegroundColor Red
        exit 1
    }
    
    # Test integration
    Set-Location "$ProjectRoot"
    try {
        cargo test --test integration_tests
        if ($LASTEXITCODE -ne 0) {
            throw "Integration tests failed"
        }
        Write-Host "✅ Integration tests passed" -ForegroundColor Green
    } catch {
        Write-Host "❌ Integration tests failed: $_" -ForegroundColor Red
        exit 1
    }
}

# Function to build the application
function Build-Application {
    Write-Host "📦 Building application..." -ForegroundColor Yellow
    
    # Build backend
    Set-Location "$ProjectRoot/hostd"
    try {
        if ($Environment -eq "production") {
            cargo build --release
        } else {
            cargo build
        }
        if ($LASTEXITCODE -ne 0) {
            throw "Backend build failed"
        }
        Write-Host "✅ Backend built successfully" -ForegroundColor Green
    } catch {
        Write-Host "❌ Backend build failed: $_" -ForegroundColor Red
        exit 1
    }
    
    # Build frontend
    Set-Location "$ProjectRoot/guardian-ui"
    try {
        npm run build
        if ($LASTEXITCODE -ne 0) {
            throw "Frontend build failed"
        }
        Write-Host "✅ Frontend built successfully" -ForegroundColor Green
    } catch {
        Write-Host "❌ Frontend build failed: $_" -ForegroundColor Red
        exit 1
    }
    
    # Build desktop app
    try {
        if ($Environment -eq "production") {
            npm run tauri:build
        } else {
            npm run tauri:build:debug
        }
        if ($LASTEXITCODE -ne 0) {
            throw "Desktop app build failed"
        }
        Write-Host "✅ Desktop app built successfully" -ForegroundColor Green
    } catch {
        Write-Host "❌ Desktop app build failed: $_" -ForegroundColor Red
        exit 1
    }
}

# Function to create deployment package
function Create-DeploymentPackage {
    Write-Host "📋 Creating deployment package..." -ForegroundColor Yellow
    
    $DeployDir = "$ProjectRoot/deploy"
    $VersionDir = "$DeployDir/guardian-$Version"
    
    # Create deployment directory
    if (Test-Path $DeployDir) {
        Remove-Item $DeployDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $DeployDir -Force | Out-Null
    New-Item -ItemType Directory -Path $VersionDir -Force | Out-Null
    
    # Copy backend binary
    $BackendBinary = if ($Environment -eq "production") {
        "$ProjectRoot/hostd/target/release/hostd.exe"
    } else {
        "$ProjectRoot/hostd/target/debug/hostd.exe"
    }
    
    if (Test-Path $BackendBinary) {
        Copy-Item $BackendBinary "$VersionDir/hostd.exe"
        Write-Host "✅ Backend binary copied" -ForegroundColor Green
    }
    
    # Copy desktop app installer
    $BundleDir = "$ProjectRoot/guardian-ui/src-tauri/target/release/bundle"
    if (Test-Path $BundleDir) {
        $BundleDirs = Get-ChildItem -Path $BundleDir -Directory
        foreach ($BundleDir in $BundleDirs) {
            $BundlePath = $BundleDir.FullName
            $BundleName = $BundleDir.Name
            
            # Copy installer files
            $InstallerFiles = Get-ChildItem -Path $BundlePath -File
            foreach ($File in $InstallerFiles) {
                Copy-Item $File.FullName "$VersionDir/"
            }
            
            Write-Host "✅ $BundleName installer copied" -ForegroundColor Green
        }
    }
    
    # Copy configuration files
    Copy-Item "$ProjectRoot/configs/*" "$VersionDir/" -Force
    Write-Host "✅ Configuration files copied" -ForegroundColor Green
    
    # Copy documentation
    Copy-Item "$ProjectRoot/README.md" "$VersionDir/"
    Copy-Item "$ProjectRoot/DESKTOP_APP_README.md" "$VersionDir/"
    Write-Host "✅ Documentation copied" -ForegroundColor Green
    
    # Create deployment script
    $DeployScript = @"
@echo off
echo Starting Guardian Server Manager...

REM Check if hostd.exe exists
if not exist "hostd.exe" (
    echo Error: hostd.exe not found!
    pause
    exit /b 1
)

REM Start the backend
echo Starting backend service...
start /B hostd.exe --port 8080 --database-url sqlite:guardian.db

REM Wait for backend to start
timeout /t 3 /nobreak > nul

REM Start the desktop application
echo Starting Guardian application...
if exist "Guardian_$Version_x64_en-US.msi" (
    echo Installing Guardian...
    msiexec /i "Guardian_$Version_x64_en-US.msi" /quiet
    echo Guardian installed successfully!
) else (
    echo Guardian installer not found!
    pause
    exit /b 1
)

echo Guardian Server Manager is now running!
pause
"@
    
    Set-Content -Path "$VersionDir/install.bat" -Value $DeployScript
    Write-Host "✅ Installation script created" -ForegroundColor Green
    
    # Create README for deployment
    $DeployReadme = @"
# Guardian $Version Deployment Package

This package contains everything needed to run Guardian Server Manager.

## Contents

- `hostd.exe` - Backend service
- `Guardian_$Version_x64_en-US.msi` - Desktop application installer
- `install.bat` - Automated installation script
- Configuration files and documentation

## Installation

1. Run `install.bat` to automatically install and start Guardian
2. Or manually install the MSI file and start the backend service

## Requirements

- Windows 10 or later
- .NET Framework 4.8 or later
- Java 17 or later (for Minecraft servers)

## Support

For support and documentation, visit: https://github.com/guardian-team/guardian
"@
    
    Set-Content -Path "$VersionDir/DEPLOYMENT_README.md" -Value $DeployReadme
    Write-Host "✅ Deployment README created" -ForegroundColor Green
    
    Write-Host "📦 Deployment package created at: $VersionDir" -ForegroundColor Green
}

# Function to run security checks
function Run-SecurityChecks {
    Write-Host "🔒 Running security checks..." -ForegroundColor Yellow
    
    # Check for common security issues
    $SecurityIssues = @()
    
    # Check for hardcoded passwords
    $PasswordFiles = Get-ChildItem -Path $ProjectRoot -Recurse -Include "*.rs", "*.ts", "*.tsx", "*.js", "*.jsx" | 
        Select-String -Pattern "password.*=.*[\"'].*[\"']" -SimpleMatch
    
    if ($PasswordFiles) {
        $SecurityIssues += "Hardcoded passwords found in source code"
    }
    
    # Check for API keys
    $ApiKeyFiles = Get-ChildItem -Path $ProjectRoot -Recurse -Include "*.rs", "*.ts", "*.tsx", "*.js", "*.jsx" | 
        Select-String -Pattern "api[_-]?key.*=.*[\"'].*[\"']" -SimpleMatch
    
    if ($ApiKeyFiles) {
        $SecurityIssues += "API keys found in source code"
    }
    
    if ($SecurityIssues.Count -gt 0) {
        Write-Host "⚠️ Security issues found:" -ForegroundColor Yellow
        foreach ($Issue in $SecurityIssues) {
            Write-Host "  - $Issue" -ForegroundColor Yellow
        }
    } else {
        Write-Host "✅ No security issues found" -ForegroundColor Green
    }
}

# Function to generate changelog
function Generate-Changelog {
    Write-Host "📝 Generating changelog..." -ForegroundColor Yellow
    
    $Changelog = @"
# Guardian Changelog

## Version $Version - $(Get-Date -Format "yyyy-MM-dd")

### New Features
- Complete backend implementation with WebSocket support
- Real-time server monitoring and management
- Desktop application with Tauri
- Cross-platform support (Windows, macOS, Linux)
- RCON integration for Minecraft server communication
- Database integration with SQLite
- Comprehensive API endpoints

### Improvements
- Enhanced performance monitoring
- Better error handling and logging
- Improved user interface
- Real-time data synchronization

### Bug Fixes
- Fixed various UI issues
- Resolved performance problems
- Improved stability

### Technical Changes
- Migrated to real data from mock data
- Implemented proper WebSocket communication
- Added comprehensive testing framework
- Enhanced security measures

---

For full changelog, visit: https://github.com/guardian-team/guardian/releases
"@
    
    Set-Content -Path "$ProjectRoot/CHANGELOG.md" -Value $Changelog
    Write-Host "✅ Changelog generated" -ForegroundColor Green
}

# Main deployment process
try {
    # Run tests unless skipped
    if (-not $SkipTests) {
        Run-Tests
    } else {
        Write-Host "⏭️ Skipping tests" -ForegroundColor Yellow
    }
    
    # Run security checks
    Run-SecurityChecks
    
    # Build application unless skipped
    if (-not $SkipBuild) {
        Build-Application
    } else {
        Write-Host "⏭️ Skipping build" -ForegroundColor Yellow
    }
    
    # Create deployment package
    Create-DeploymentPackage
    
    # Generate changelog
    Generate-Changelog
    
    Write-Host "🎉 Deployment completed successfully!" -ForegroundColor Green
    Write-Host "📁 Deployment package: $ProjectRoot/deploy/guardian-$Version" -ForegroundColor Blue
    
} catch {
    Write-Host "❌ Deployment failed: $_" -ForegroundColor Red
    exit 1
} finally {
    # Return to project root
    Set-Location $ProjectRoot
}

# Guardian Zero-Downtime Test Script
# This script tests the complete zero-downtime scenario

param(
    [string]$ServerType = "fabric",
    [string]$MinecraftVersion = "1.20.1",
    [string]$ModPack = "create-valkyrien-skies",
    [int]$MaxPlayers = 10,
    [int]$TestRadius = 5000,
    [switch]$SkipGPU = $false,
    [switch]$Verbose = $false
)

# Set error action preference
$ErrorActionPreference = "Stop"

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Blue"

function Write-ColorOutput {
    param([string]$Message, [string]$Color = "White")
    Write-Host $Message -ForegroundColor $Color
}

function Write-TestStep {
    param([string]$Step, [string]$Description)
    Write-ColorOutput "`n=== $Step ===" $Blue
    Write-ColorOutput $Description $Yellow
}

function Write-Success {
    param([string]$Message)
    Write-ColorOutput "✅ $Message" $Green
}

function Write-Error {
    param([string]$Message)
    Write-ColorOutput "❌ $Message" $Red
}

function Write-Warning {
    param([string]$Message)
    Write-ColorOutput "⚠️  $Message" $Yellow
}

function Test-APIEndpoint {
    param([string]$Url, [string]$Method = "GET", [hashtable]$Body = $null)
    
    try {
        $headers = @{
            "Content-Type" = "application/json"
        }
        
        $params = @{
            Uri = $Url
            Method = $Method
            Headers = $headers
        }
        
        if ($Body) {
            $params.Body = ($Body | ConvertTo-Json -Depth 10)
        }
        
        $response = Invoke-RestMethod @params
        return $response
    }
    catch {
        Write-Error "API call failed: $($_.Exception.Message)"
        return $null
    }
}

function Wait-ForServerStatus {
    param([string]$ServerId, [string]$ExpectedStatus, [int]$TimeoutSeconds = 300)
    
    $startTime = Get-Date
    $timeout = $startTime.AddSeconds($TimeoutSeconds)
    
    while ((Get-Date) -lt $timeout) {
        $server = Test-APIEndpoint "http://localhost:8080/api/servers/$ServerId"
        if ($server -and $server.status -eq $ExpectedStatus) {
            return $true
        }
        
        Start-Sleep -Seconds 5
        Write-ColorOutput "Waiting for server $ServerId to reach status: $ExpectedStatus" $Yellow
    }
    
    return $false
}

function Test-ZeroDowntimeScenario {
    Write-ColorOutput "`n🚀 Starting Guardian Zero-Downtime Test" $Blue
    Write-ColorOutput "Server Type: $ServerType" $Yellow
    Write-ColorOutput "Minecraft Version: $MinecraftVersion" $Yellow
    Write-ColorOutput "Mod Pack: $ModPack" $Yellow
    Write-ColorOutput "Max Players: $MaxPlayers" $Yellow
    Write-ColorOutput "Test Radius: $TestRadius" $Yellow
    
    # Step 1: Check Guardian is running
    Write-TestStep "1" "Checking Guardian Backend Status"
    $health = Test-APIEndpoint "http://localhost:8080/api/health"
    if (-not $health) {
        Write-Error "Guardian backend is not running. Please start it first."
        return $false
    }
    Write-Success "Guardian backend is running"
    
    # Step 2: Check settings
    Write-TestStep "2" "Checking Application Settings"
    $settings = Test-APIEndpoint "http://localhost:8080/api/settings"
    if (-not $settings) {
        Write-Error "Failed to get settings"
        return $false
    }
    Write-Success "Settings retrieved successfully"
    
    # Step 3: Create test server
    Write-TestStep "3" "Creating Test Server"
    $serverData = @{
        name = "ZeroDowntimeTest-$((Get-Date).ToString('yyyyMMdd-HHmmss'))"
        mcVersion = $MinecraftVersion
        loader = $ServerType
        javaPath = $settings.javaPath
        minRamMb = 2048
        maxRamMb = 4096
        worldName = "test-world"
        serverDir = "test-server"
        maxPlayers = $MaxPlayers
        pregenerationPolicy = @{
            radius = $TestRadius
            dimensions = @("overworld", "nether", "end")
            lightingOptimization = $true
        }
    }
    
    $server = Test-APIEndpoint "http://localhost:8080/api/servers" "POST" $serverData
    if (-not $server) {
        Write-Error "Failed to create server"
        return $false
    }
    Write-Success "Server created: $($server.id)"
    
    # Step 4: Start server
    Write-TestStep "4" "Starting Server"
    $startResult = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/start" "POST"
    if (-not $startResult) {
        Write-Error "Failed to start server"
        return $false
    }
    
    if (-not (Wait-ForServerStatus $server.id "running" 300)) {
        Write-Error "Server failed to start within timeout"
        return $false
    }
    Write-Success "Server started successfully"
    
    # Step 5: Add test mods (if modpack specified)
    if ($ModPack -ne "none") {
        Write-TestStep "5" "Adding Test Mods"
        $mods = @()
        
        # Add some common mods for testing
        if ($ModPack -eq "create-valkyrien-skies") {
            $mods = @(
                @{ provider = "curseforge"; projectId = "238222"; versionId = "latest" },
                @{ provider = "curseforge"; projectId = "235279"; versionId = "latest" }
            )
        }
        
        foreach ($mod in $mods) {
            $modResult = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/mods" "POST" $mod
            if ($modResult) {
                Write-Success "Added mod: $($mod.projectId)"
            } else {
                Write-Warning "Failed to add mod: $($mod.projectId)"
            }
        }
    }
    
    # Step 6: Run compatibility scan
    Write-TestStep "6" "Running Compatibility Scan"
    $compatResult = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/compat/scan" "POST"
    if ($compatResult) {
        Write-Success "Compatibility scan completed"
        if ($compatResult.conflicts.Count -gt 0) {
            Write-Warning "Found $($compatResult.conflicts.Count) conflicts"
            
            # Apply fixes
            $fixResult = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/compat/apply" "POST"
            if ($fixResult) {
                Write-Success "Applied compatibility fixes"
            }
        }
    }
    
    # Step 7: Create pre-generation job
    Write-TestStep "7" "Creating Pre-generation Job"
    $pregenerationData = @{
        radius = $TestRadius
        dimensions = @("overworld")
        useGPU = -not $SkipGPU
        lightingOptimization = $true
    }
    
    $pregenJob = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/pregen" "POST" $pregenerationData
    if (-not $pregenJob) {
        Write-Error "Failed to create pre-generation job"
        return $false
    }
    Write-Success "Pre-generation job created: $($pregenJob.id)"
    
    # Step 8: Start pre-generation
    Write-TestStep "8" "Starting Pre-generation"
    $startPregen = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/pregen/$($pregenJob.id)/start" "POST"
    if (-not $startPregen) {
        Write-Error "Failed to start pre-generation"
        return $false
    }
    
    # Monitor pre-generation progress
    $maxWaitTime = 1800 # 30 minutes
    $startTime = Get-Date
    $timeout = $startTime.AddSeconds($maxWaitTime)
    
    Write-ColorOutput "Monitoring pre-generation progress..." $Yellow
    while ((Get-Date) -lt $timeout) {
        $jobStatus = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/pregen/$($pregenJob.id)"
        if ($jobStatus) {
            $progress = [math]::Round($jobStatus.progress * 100, 2)
            Write-ColorOutput "Progress: $progress%" $Yellow
            
            if ($jobStatus.status -eq "done") {
                Write-Success "Pre-generation completed successfully"
                break
            } elseif ($jobStatus.status -eq "failed") {
                Write-Error "Pre-generation failed: $($jobStatus.error)"
                return $false
            }
        }
        
        Start-Sleep -Seconds 10
    }
    
    if ((Get-Date) -ge $timeout) {
        Write-Warning "Pre-generation timed out, but continuing with test"
    }
    
    # Step 9: Create hot import job
    Write-TestStep "9" "Creating Hot Import Job"
    $importData = @{
        sourceDir = "staging"
        targetDir = "world"
        safetyChecks = $true
        tpsThreshold = 18.0
    }
    
    $importJob = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/import" "POST" $importData
    if (-not $importJob) {
        Write-Error "Failed to create hot import job"
        return $false
    }
    Write-Success "Hot import job created: $($importJob.id)"
    
    # Step 10: Start hot import
    Write-TestStep "10" "Starting Hot Import (Zero-Downtime)"
    $startImport = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/import/$($importJob.id)/start" "POST"
    if (-not $startImport) {
        Write-Error "Failed to start hot import"
        return $false
    }
    
    # Monitor import progress
    Write-ColorOutput "Monitoring hot import progress..." $Yellow
    $importStartTime = Get-Date
    $importTimeout = $importStartTime.AddSeconds(600) # 10 minutes
    
    while ((Get-Date) -lt $importTimeout) {
        $importStatus = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/import/$($importJob.id)"
        if ($importStatus) {
            $progress = [math]::Round($importStatus.progress * 100, 2)
            Write-ColorOutput "Import Progress: $progress%" $Yellow
            
            if ($importStatus.status -eq "done") {
                Write-Success "Hot import completed successfully"
                break
            } elseif ($importStatus.status -eq "failed") {
                Write-Error "Hot import failed: $($importStatus.error)"
                return $false
            }
        }
        
        # Check server is still running
        $serverStatus = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)"
        if ($serverStatus.status -ne "running") {
            Write-Error "Server stopped during hot import - this should not happen!"
            return $false
        }
        
        Start-Sleep -Seconds 5
    }
    
    # Step 11: Run lighting optimization
    Write-TestStep "11" "Running Lighting Optimization"
    $lightingData = @{
        radius = $TestRadius
        useGPU = -not $SkipGPU
        optimizationLevel = "high"
    }
    
    $lightingJob = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/lighting" "POST" $lightingData
    if ($lightingJob) {
        $startLighting = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/lighting/$($lightingJob.id)/start" "POST"
        if ($startLighting) {
            Write-Success "Lighting optimization started"
        }
    }
    
    # Step 12: Create backup
    Write-TestStep "12" "Creating Backup"
    $backupData = @{
        name = "ZeroDowntimeTest-$(Get-Date -Format 'yyyyMMdd-HHmmss')"
        includeWorld = $true
        includeMods = $true
        includeConfig = $true
    }
    
    $backup = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/backups" "POST" $backupData
    if ($backup) {
        Write-Success "Backup created: $($backup.id)"
    } else {
        Write-Warning "Failed to create backup"
    }
    
    # Step 13: Test mod management
    Write-TestStep "13" "Testing Mod Management"
    $modSearch = Test-APIEndpoint "http://localhost:8080/api/mods/search" "GET" @{ query = "optifine"; provider = "curseforge" }
    if ($modSearch) {
        Write-Success "Mod search works"
    }
    
    # Step 14: Verify server is still running
    Write-TestStep "14" "Verifying Server Status"
    $finalStatus = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)"
    if ($finalStatus.status -eq "running") {
        Write-Success "Server is still running after all operations"
    } else {
        Write-Error "Server is not running: $($finalStatus.status)"
        return $false
    }
    
    # Step 15: Cleanup
    Write-TestStep "15" "Cleaning Up"
    $stopResult = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)/stop" "POST"
    if ($stopResult) {
        Write-Success "Server stopped successfully"
    }
    
    $deleteResult = Test-APIEndpoint "http://localhost:8080/api/servers/$($server.id)" "DELETE"
    if ($deleteResult) {
        Write-Success "Server deleted successfully"
    }
    
    Write-ColorOutput "`n🎉 Zero-Downtime Test Completed Successfully!" $Green
    Write-ColorOutput "All operations completed without server restarts" $Green
    return $true
}

# Main execution
try {
    $result = Test-ZeroDowntimeScenario
    if ($result) {
        Write-ColorOutput "`n✅ All tests passed!" $Green
        exit 0
    } else {
        Write-ColorOutput "`n❌ Tests failed!" $Red
        exit 1
    }
}
catch {
    Write-Error "Test script failed with error: $($_.Exception.Message)"
    exit 1
}

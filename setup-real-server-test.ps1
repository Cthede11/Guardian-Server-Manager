# Setup Real Minecraft Server Testing
# This script prepares the environment for testing with real Minecraft servers

Write-Host "Setting up Real Minecraft Server Testing..." -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green

# Create test server directory
$testServerDir = "test-minecraft-server"
if (Test-Path $testServerDir) {
    Remove-Item -Recurse -Force $testServerDir
}
New-Item -ItemType Directory -Path $testServerDir -Force
Write-Host "Created test server directory: $testServerDir" -ForegroundColor Green

# Download Minecraft server JAR
Write-Host "Downloading Minecraft server JAR..." -ForegroundColor Yellow
$serverJar = "$testServerDir\server.jar"
$serverUrl = "https://launcher.mojang.com/v1/objects/5b868151bd02b41319f54c8e5c1e8b0c5e9a97a0/server.jar"

try {
    Invoke-WebRequest -Uri $serverUrl -OutFile $serverJar
    Write-Host "Minecraft server JAR downloaded successfully" -ForegroundColor Green
} catch {
    Write-Host "Failed to download server JAR: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "You can manually download a server JAR and place it in $testServerDir" -ForegroundColor Yellow
}

# Create server.properties
Write-Host "Creating server configuration..." -ForegroundColor Yellow
$serverProperties = @"
#Minecraft server properties
server-port=25565
gamemode=survival
difficulty=easy
max-players=20
online-mode=false
white-list=false
motd=Guardian Test Server
"@

$serverProperties | Out-File -FilePath "$testServerDir\server.properties" -Encoding UTF8
Write-Host "Server properties created" -ForegroundColor Green

# Create eula.txt
Write-Host "Creating EULA agreement..." -ForegroundColor Yellow
$eula = "eula=true"
$eula | Out-File -FilePath "$testServerDir\eula.txt" -Encoding UTF8
Write-Host "EULA agreement created" -ForegroundColor Green

# Create test world directory
Write-Host "Creating test world directory..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path "$testServerDir\world" -Force | Out-Null
Write-Host "Test world directory created" -ForegroundColor Green

# Create test configuration for Guardian
Write-Host "Creating Guardian test configuration..." -ForegroundColor Yellow
$guardianConfig = @{
    name = "Real Test Server"
    type = "vanilla"
    version = "1.20.1"
    paths = @{
        world = (Resolve-Path $testServerDir).Path
    }
    java_path = "java"
    memory = "2G"
    jvm_args = "-Xmx2G -Xms1G -XX:+UseG1GC"
} | ConvertTo-Json -Depth 3

$guardianConfig | Out-File -FilePath "test-server-config.json" -Encoding UTF8
Write-Host "Guardian test configuration created" -ForegroundColor Green

Write-Host "===============================================" -ForegroundColor Green
Write-Host "Real server testing setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Test server location: $testServerDir" -ForegroundColor Cyan
Write-Host "Configuration file: test-server-config.json" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Run: .\build-production-final.ps1" -ForegroundColor White
Write-Host "2. Run: .\test-production-app.ps1" -ForegroundColor White
Write-Host "3. Test server creation with real Minecraft server" -ForegroundColor White
Write-Host "4. Test server management and monitoring" -ForegroundColor White
Write-Host ""
Write-Host "Tips for real server testing:" -ForegroundColor Yellow
Write-Host "- Make sure Java is installed and accessible" -ForegroundColor White
Write-Host "- Test with different Minecraft versions" -ForegroundColor White
Write-Host "- Test server start/stop/restart functionality" -ForegroundColor White
Write-Host "- Test performance monitoring and metrics" -ForegroundColor White
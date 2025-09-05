# Test Backend Connection
Write-Host "Testing Guardian Backend Connection..." -ForegroundColor Cyan

# Test health endpoint
Write-Host "Testing health endpoint..." -ForegroundColor Yellow
try {
    $healthResponse = Invoke-RestMethod -Uri "http://localhost:8080/api/health" -Method GET
    Write-Host "Health check response: $($healthResponse | ConvertTo-Json)" -ForegroundColor Green
} catch {
    Write-Host "Health check failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test servers endpoint
Write-Host "Testing servers endpoint..." -ForegroundColor Yellow
try {
    $serversResponse = Invoke-RestMethod -Uri "http://localhost:8080/api/servers" -Method GET
    Write-Host "Servers response: $($serversResponse | ConvertTo-Json)" -ForegroundColor Green
} catch {
    Write-Host "Servers endpoint failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test creating a server
Write-Host "Testing server creation..." -ForegroundColor Yellow
$serverData = @{
    name = "Test Server"
    loader = "vanilla"
    version = "1.21.1"
    paths = @{
        world = "./test-world"
        mods = "./test-mods"
        config = "./test-config"
    }
} | ConvertTo-Json

try {
    $createResponse = Invoke-RestMethod -Uri "http://localhost:8080/api/servers" -Method POST -Body $serverData -ContentType "application/json"
    Write-Host "Server creation response: $($createResponse | ConvertTo-Json)" -ForegroundColor Green
} catch {
    Write-Host "Server creation failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Response: $($_.Exception.Response)" -ForegroundColor Red
}

Write-Host "Backend test complete!" -ForegroundColor Cyan

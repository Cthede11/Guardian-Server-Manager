# Debug API calls for server creation
Write-Host "Testing Guardian API..." -ForegroundColor Green

# Test health endpoint
Write-Host "`n1. Testing health endpoint..." -ForegroundColor Yellow
try {
    $healthResponse = Invoke-RestMethod -Uri "http://localhost:8080/api/health" -Method GET
    Write-Host "Health check: $($healthResponse.data)" -ForegroundColor Green
} catch {
    Write-Host "Health check failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test servers endpoint
Write-Host "`n2. Testing servers endpoint..." -ForegroundColor Yellow
try {
    $serversResponse = Invoke-RestMethod -Uri "http://localhost:8080/api/servers" -Method GET
    Write-Host "Current servers: $($serversResponse.data.Count)" -ForegroundColor Green
    if ($serversResponse.data.Count -gt 0) {
        Write-Host "First server: $($serversResponse.data[0].name)" -ForegroundColor Cyan
    }
} catch {
    Write-Host "Servers endpoint failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test server creation
Write-Host "`n3. Testing server creation..." -ForegroundColor Yellow
$serverData = @{
    name = "Test Server Debug"
    loader = "vanilla"
    version = "1.21.1"
    paths = @{
        world = "./world"
        mods = "./mods"
        config = "./config"
    }
} | ConvertTo-Json -Depth 3

try {
    $createResponse = Invoke-RestMethod -Uri "http://localhost:8080/api/servers" -Method POST -Body $serverData -ContentType "application/json"
    Write-Host "Server creation successful!" -ForegroundColor Green
    Write-Host "Created server: $($createResponse.data.name) (ID: $($createResponse.data.id))" -ForegroundColor Cyan
} catch {
    Write-Host "Server creation failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
        Write-Host "Response body: $responseBody" -ForegroundColor Red
    }
}

Write-Host "`nAPI test complete!" -ForegroundColor Green

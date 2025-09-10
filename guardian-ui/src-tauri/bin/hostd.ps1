# Simple test hostd script for development
Write-Host "Starting test hostd on port 52100..."

# Create a simple health endpoint response
$healthResponse = @{
    ok = $true
    port = 52100
    pid = [System.Diagnostics.Process]::GetCurrentProcess().Id
} | ConvertTo-Json

Write-Host "Health endpoint: http://127.0.0.1:52100/healthz"
Write-Host "Response: $healthResponse"

# Keep the script running
while ($true) {
    Start-Sleep -Seconds 1
}

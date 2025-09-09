$BASE = "http://127.0.0.1:8080"
for ($p = 8080; $p -le 8090; $p++) {
  try {
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:$p/healthz" -UseBasicParsing -TimeoutSec 1
    if ($response.StatusCode -eq 200) {
      $BASE = "http://127.0.0.1:$p"
      break
    }
  } catch {
    # Continue to next port
  }
}

Write-Host "Using $BASE"
Invoke-WebRequest -Uri "$BASE/healthz" -UseBasicParsing
try { Invoke-WebRequest -Uri "$BASE/api/servers/test/metrics" -UseBasicParsing } catch { Invoke-WebRequest -Uri "$BASE/servers/test/metrics" -UseBasicParsing }
try { Invoke-WebRequest -Uri "$BASE/api/servers/test/world" -UseBasicParsing } catch { Invoke-WebRequest -Uri "$BASE/servers/test/world" -UseBasicParsing }
Invoke-WebRequest -Uri "$BASE/api/servers/test/pregen/plan" -Method POST -ContentType "application/json" -Body '{"radius":1000,"dimensions":["minecraft:overworld"]}' -UseBasicParsing
Invoke-WebRequest -Uri "$BASE/api/servers/test/pregen/start" -Method POST -UseBasicParsing
Invoke-WebRequest -Uri "$BASE/api/servers/test/pregen/status" -UseBasicParsing

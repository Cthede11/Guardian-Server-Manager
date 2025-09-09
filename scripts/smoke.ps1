$min=52100; $max=52150
$healthy=$null
for ($p=$min; $p -le $max; $p++) {
  try { $r=Invoke-WebRequest -UseBasicParsing "http://127.0.0.1:$p/healthz" -TimeoutSec 1
        if ($r.StatusCode -eq 200) { $healthy=$p; break } } catch {}
}
if (-not $healthy) { throw "No healthy hostd in $min-$max" }
Write-Host "Using port $healthy"
Invoke-WebRequest -UseBasicParsing "http://127.0.0.1:$healthy/api/servers/test/world" | Out-Null
Invoke-WebRequest -UseBasicParsing "http://127.0.0.1:$healthy/api/servers/test/pregen/status" | Out-Null
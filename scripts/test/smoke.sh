#!/usr/bin/env bash
set -euo pipefail
BASE="http://127.0.0.1:8080"
for p in $(seq 8080 8090); do
  if curl -fsS "http://127.0.0.1:$p/healthz" >/dev/null; then BASE="http://127.0.0.1:$p"; break; fi
done

echo "Using $BASE"
curl -fsS "$BASE/healthz"
curl -fsS "$BASE/api/servers/test/metrics" || curl -fsS "$BASE/servers/test/metrics"
curl -fsS "$BASE/api/servers/test/world"   || curl -fsS "$BASE/servers/test/world"
curl -N --max-time 2 "$BASE/api/servers/test/metrics/stream" >/dev/null || true
curl -fsS -X POST "$BASE/api/servers/test/pregen/plan"   -H "content-type: application/json" -d '{"radius":1000,"dimensions":["minecraft:overworld"]}'
curl -fsS -X POST "$BASE/api/servers/test/pregen/start"
curl -fsS "$BASE/api/servers/test/pregen/status"

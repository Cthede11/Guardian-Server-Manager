import { API_BASE } from "../../lib/api";

export type MetricsPoint = {
  timestamp: number; tps: number; tick_p95_ms: number;
  heap_mb?: number | null; gpu_latency_ms?: number | null;
};

export function subscribeMetrics(serverId: string, onPoint: (p: MetricsPoint)=>void) {
  const es = new EventSource(`${API_BASE}/servers/${serverId}/metrics/stream`);
  es.onmessage = (ev) => { try { onPoint(JSON.parse(ev.data)); } catch {} };
  es.onerror   = () => { /* Let UI show 'not available' gracefully */ };
  return () => es.close();
}

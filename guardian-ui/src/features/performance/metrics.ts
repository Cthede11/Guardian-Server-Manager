import { getAPI_BASE } from "../../lib/api";
export type MetricsPoint = { timestamp: number; tps: number; tick_p95_ms: number; heap_mb?: number|null; gpu_latency_ms?: number|null; };

export async function subscribeMetrics(serverId: string, onPoint: (p: MetricsPoint)=>void) {
  const base = await getAPI_BASE();
  const es = new EventSource(`${base}/api/servers/${serverId}/metrics/stream`);
  es.onmessage = (ev) => { try { onPoint(JSON.parse(ev.data)); } catch {} };
  es.onerror   = () => { /* show small 'stream disconnected' banner, keep history */ };
  return () => es.close();
}

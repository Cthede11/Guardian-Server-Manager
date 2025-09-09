import { useEffect, useState } from "react";
import { apiClient as api } from "../../lib/api";

type PregenStatus = {
  state: "idle"|"running"|"paused"|"failed"|"done";
  progress: number;
  eta_seconds: number|null;
  capabilities: { gpu: boolean; fallback: boolean };
  last_error: string|null;
};

export default function PregenPage({ serverId }: { serverId: string }) {
  const [s, setS] = useState<PregenStatus | null>(null);
  const [err, setErr] = useState<string | null>(null);

  const refresh = async () => {
    try {
      setS(await api.call(`/servers/${serverId}/pregen/status`));
      setErr(null);
    } catch (e: any) {
      setErr(e?.message ?? "unknown");
    }
  };

  useEffect(() => {
    refresh();
    const t = setInterval(refresh, 2000);
    return () => clearInterval(t);
  }, [serverId]);

  if (err) return <div className="p-6">Pregen unavailable: {String(err)}</div>;
  if (!s) return <div className="p-6">Loading pregen status…</div>;

  const handleAction = async (action: string) => {
    try {
      await api.call(`/servers/${serverId}/pregen/${action}`, { method: "POST" });
      await refresh();
    } catch (e: any) {
      setErr(e?.message ?? "unknown");
    }
  };

  return (
    <div className="p-6 space-y-3">
      {!s.capabilities?.gpu && (
        <div className="text-yellow-500">
          GPU worker not available—using server-driven fallback.
        </div>
      )}
      <div>
        State: <b>{s.state}</b> • Progress: {(s.progress*100).toFixed(1)}%{" "}
        {s.eta_seconds ? `(ETA ${Math.round(s.eta_seconds)}s)` : ""}
      </div>
      <div className="space-x-2">
        <button onClick={() => handleAction("start")}>Start</button>
        <button onClick={() => handleAction("pause")}>Pause</button>
        <button onClick={() => handleAction("resume")}>Resume</button>
        <button onClick={() => handleAction("cancel")}>Cancel</button>
      </div>
      {s.last_error && (
        <div className="text-red-500">Last error: {s.last_error}</div>
      )}
    </div>
  );
}

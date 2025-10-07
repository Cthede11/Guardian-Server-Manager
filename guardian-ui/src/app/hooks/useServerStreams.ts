import { useEffect } from "react";
import { events } from "@/lib/client";
import { useLive } from "@/store/live-new";
import type { ConsoleLines, Metrics, Player, FreezeTicket, PregenJob, ServerHealth } from "@/lib/types.gen";

export function useServerStreams(serverId?: string) {
  const appendConsole = useLive((state) => state.appendConsole);
  const updatePlayers = useLive((state) => state.updatePlayers);
  const updateFreezes = useLive((state) => state.updateFreezes);
  const applyMetrics = useLive((state) => state.applyMetrics);
  const updatePregenJobs = useLive((state) => state.updatePregenJobs);
  const updateHealth = useLive((state) => state.updateHealth);

  useEffect(() => {
    if (!serverId) return;

    const unsubs: Array<() => void> = [];

    const setupSubscriptions = async () => {
      try {
        // Console stream
        unsubs.push(
          await events.subscribeToConsole(serverId, (payload: ConsoleLines) => {
            const consoleMessages = payload.map(line => ({
              ts: new Date(line.timestamp).getTime(),
              level: line.level,
              msg: line.message
            }));
            appendConsole(serverId, consoleMessages);
          })
        );

        // Metrics stream
        unsubs.push(
          await events.subscribeToMetrics(serverId, (payload: Metrics) => {
            applyMetrics(serverId, {
              tps: payload.tps,
              tick_p95_ms: payload.tickP95,
              heap_mb: payload.heapMb,
              gpu_queue_ms: payload.gpuQueueMs,
              players_online: payload.playersOnline
            });
          })
        );

        // Players stream
        unsubs.push(
          await events.subscribeToPlayers(serverId, (payload: Player[]) => {
            updatePlayers(serverId, payload);
          })
        );

        // Freezes stream
        unsubs.push(
          await events.subscribeToFreezes(serverId, (payload: FreezeTicket[]) => {
            updateFreezes(serverId, payload);
          })
        );

        // Pregen jobs stream
        unsubs.push(
          await events.subscribeToPregen(serverId, (payload: PregenJob[]) => {
            updatePregenJobs(serverId, payload);
          })
        );

        // Health stream
        unsubs.push(
          await events.subscribeToHealth(serverId, (payload: ServerHealth) => {
            updateHealth(serverId, payload);
          })
        );
      } catch (error) {
        console.error('Failed to setup server streams:', error);
      }
    };

    setupSubscriptions();

    return () => {
      unsubs.forEach(unsub => unsub());
    };
  }, [serverId, appendConsole, updatePlayers, updateFreezes, applyMetrics, updatePregenJobs, updateHealth]);
}

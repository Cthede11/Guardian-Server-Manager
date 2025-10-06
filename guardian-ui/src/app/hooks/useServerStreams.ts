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
            appendConsole(serverId, payload);
          })
        );

        // Metrics stream
        unsubs.push(
          await events.subscribeToMetrics(serverId, (payload: Metrics) => {
            applyMetrics(serverId, payload);
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

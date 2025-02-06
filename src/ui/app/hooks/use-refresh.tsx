import { useAppState, refresh } from "@/lib/state";
import { useCallback } from "react";

export function useRefresh() {
  const lastRefresh = useAppState((state) => state.lastRefresh);
  // TODO: queue a debounced refresh?
  const handleStale = useCallback(
    function <T>(data: (T | undefined)[]) {
      return data.filter((x) => x !== undefined);
    },
    [lastRefresh],
  );

  return {
    refreshToken: lastRefresh,
    refresh,
    handleStaleApps: handleStale,
    handleStaleTags: handleStale,
    handleStaleAlerts: handleStale,
  };
}

import { useAppState, refresh } from "@/lib/state";

export function useRefresh() {
  const lastRefresh = useAppState((state) => state.lastRefresh);

  return { refreshToken: lastRefresh, refresh };
}

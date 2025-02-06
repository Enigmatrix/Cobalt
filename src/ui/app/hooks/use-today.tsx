import { useAppState } from "@/lib/state";
import { useMemo } from "react";

export function useToday() {
  const lastRefresh = useAppState((state) => state.lastRefresh);
  const today = useMemo(() => {
    return lastRefresh.startOf("day");
  }, [+lastRefresh.startOf("day").toJSDate()]); // memoize today
  return today;
}

export function getToday() {
  const lastRefresh = useAppState.getState().lastRefresh;
  return lastRefresh.startOf("day");
}

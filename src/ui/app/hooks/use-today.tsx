import { useAppState } from "@/lib/state";
import { useMemo } from "react";

export function useToday() {
  const lastRefresh = useAppState((state) => state.lastRefresh);
  const today = useMemo(() => {
    return lastRefresh.startOf("day");
    // lastRefresh is a DateTime object, so we can't compare it directly
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [+lastRefresh.startOf("day")]); // memoize today
  return today;
}

export function getToday() {
  const lastRefresh = useAppState.getState().lastRefresh;
  return lastRefresh.startOf("day");
}

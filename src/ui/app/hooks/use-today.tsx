import { useAppState } from "@/lib/state";
import { Interval as LuxonInterval, type DateTime } from "luxon";
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

export type TimePeriod = "day" | "week" | "month" | "year";

// Like Luxon's Interval type, but not half-open.
export interface Interval {
  start: DateTime;
  end: DateTime;
}

export function useTimePeriod(unit: TimePeriod): Interval {
  const today = useToday();
  const luxonInterval = LuxonInterval.after(today.startOf(unit), { [unit]: 1 });
  return { start: luxonInterval.start!, end: luxonInterval.end! };
}

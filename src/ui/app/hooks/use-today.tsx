import { useAppState } from "@/lib/state";
import type { TimePeriod } from "@/lib/time";
import { Interval as LuxonInterval } from "luxon";
import type { Interval } from "@/lib/time";
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

export function useTimePeriod(unit: TimePeriod): Interval {
  const today = useToday();
  const range = useMemo(() => {
    const luxonInterval = LuxonInterval.after(today.startOf(unit), {
      [unit]: 1,
    });
    return { start: luxonInterval.start!, end: luxonInterval.end! };
  }, [today, unit]);
  return range;
}

import { useAppState } from "@/lib/state";
import { Interval as LuxonInterval } from "luxon";
import type { Interval } from "@/lib/time";
import { useMemo, useCallback, useState } from "react";
import type { Period } from "@/lib/entities";

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

export function usePeriodInterval(unit: Period): Interval {
  const today = useToday();
  const interval = useMemo(() => {
    const luxonInterval = LuxonInterval.after(today.startOf(unit), {
      [unit]: 1,
    });
    return { start: luxonInterval.start!, end: luxonInterval.end! };
  }, [today, unit]);
  return interval;
}

export interface IntervalControls {
  canGoNext: boolean;
  goNext: () => void;
  canGoPrev: boolean;
  goPrev: () => void;
}

export function useIntervalControls(
  interval: Interval | null,
  setInterval: (interval: Interval | null) => void,
): IntervalControls {
  const today = useToday();

  const duration = useMemo(() => {
    return interval?.end.diff(interval.start, [
      "years",
      "months",
      "days",
      "hours",
      "minutes",
      "seconds",
      "milliseconds",
    ]);
  }, [interval]);
  const canGoNext = useMemo(
    () =>
      interval !== null &&
      duration !== undefined &&
      today.plus({ day: 1 }) > interval.start.plus(duration),
    [interval, today, duration],
  );
  // can always go prev, as long as we have an interval
  const canGoPrev = useMemo(() => interval !== null, [interval]);

  const goNext = useCallback(() => {
    if (!interval || !duration) return;
    setInterval({
      start: interval.start.plus(duration),
      end: interval.end.plus(duration),
    });
  }, [interval, duration, setInterval]);
  const goPrev = useCallback(() => {
    if (!interval || !duration) return;
    setInterval({
      start: interval.start.minus(duration),
      end: interval.end.minus(duration),
    });
  }, [interval, duration, setInterval]);

  return { canGoNext, goNext, canGoPrev, goPrev };
}

export function useIntervalControlsWithDefault(
  initialPeriod: Period,
): IntervalControls & {
  interval: Interval | null;
  setInterval: (interval: Interval | null) => void;
} {
  const initialInterval = usePeriodInterval(initialPeriod);
  const [interval, setInterval] = useState<Interval | null>(initialInterval);
  const controls = useIntervalControls(interval, setInterval);
  return { ...controls, interval, setInterval };
}

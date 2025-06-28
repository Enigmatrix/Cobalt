import { useTargetApps } from "@/hooks/use-refresh";
import { useAppDurations } from "@/hooks/use-repo";
import { usePeriodInterval } from "@/hooks/use-time";
import { timeFrameToPeriod, type Target, type TimeFrame } from "@/lib/entities";
import { useMemo } from "react";

export interface TriggerInfo {
  currentUsage: number;
  alert: boolean;
  reminders: { id?: number; trigger: boolean }[];
}

export function useTriggerInfo(
  target?: Target,
  usageLimit?: number,
  timeFrame?: TimeFrame,
  reminders?: { id?: number; threshold: number; message: string }[],
): TriggerInfo {
  const interval = usePeriodInterval(timeFrameToPeriod(timeFrame ?? "daily"));

  const targetApps = useTargetApps(target);
  const { ret: appDurations } = useAppDurations({
    start: interval.start,
    end: interval.end,
  });
  const totalUsage = useMemo(() => {
    if (!targetApps) return 0;
    return targetApps.reduce(
      (acc, curr) => acc + (appDurations[curr.id]?.duration ?? 0),
      0,
    );
  }, [appDurations, targetApps]);

  const triggerInfo = useMemo(() => {
    if (!usageLimit || !timeFrame || !target)
      return { alert: false, reminders: [], currentUsage: totalUsage };

    const alert = totalUsage > usageLimit;
    const reminderTriggers = (reminders ?? []).map((reminder) => ({
      id: reminder.id,
      trigger: totalUsage > reminder.threshold * usageLimit,
    }));
    return { alert, reminders: reminderTriggers, currentUsage: totalUsage };
  }, [totalUsage, target, timeFrame, usageLimit, reminders]);

  return triggerInfo;
}

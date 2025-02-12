import type { App, Ref, WithGroupedDuration } from "@/lib/entities";
import { getAppDurationsPerPeriod } from "@/lib/repo";
import type { EntityMap } from "@/lib/state";
import _ from "lodash";
import type { DateTime, Duration } from "luxon";
import { useEffect, useState, useTransition } from "react";
import { useRefresh } from "@/hooks/use-refresh";

export function useAppDurationsPerPeriod({
  start,
  end,
  period,
  appId,
}: {
  start?: DateTime;
  end?: DateTime;
  period?: Duration;
  appId?: Ref<App>;
}) {
  const { refreshToken } = useRefresh();
  const [ret, setRet] = useState<{
    appUsage: number;
    totalUsage: number;
    usages: EntityMap<App, WithGroupedDuration<App>[]>;
    start?: DateTime;
    end?: DateTime;
    period?: Duration;
  }>({ appUsage: 0, totalUsage: 0, usages: {}, start, end, period });
  const [isLoading, startTransition] = useTransition();
  useEffect(() => {
    startTransition(async () => {
      if (!start || !end || !period) return;

      const usages = await getAppDurationsPerPeriod({
        start,
        end,
        period,
      });
      const appUsage = appId ? _(usages[appId]).sumBy("duration") : 0;
      const totalUsage = _(usages).values().flatten().sumBy("duration");
      setRet({
        appUsage,
        totalUsage,
        usages,
        start,
        end,
        period,
      });
    });
  }, [start, end, period, appId, refreshToken, startTransition]);
  return { ...ret, isLoading };
}

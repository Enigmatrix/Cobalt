import type { App, Ref, Tag, WithGroupedDuration } from "@/lib/entities";
import {
  getAppDurationsPerPeriod,
  getAppSessionUsages,
  getInteractionPeriods,
  getSystemEvents,
  getTagDurationsPerPeriod,
} from "@/lib/repo";
import type { EntityMap } from "@/lib/state";
import _ from "lodash";
import type { DateTime, Duration } from "luxon";
import { useEffect, useState, useTransition } from "react";
import { useRefresh } from "@/hooks/use-refresh";

export function useRepo<T, Arg extends object>(
  fn: (args: Arg) => Promise<T>,
  def: T,
  arg: Arg,
) {
  const { refreshToken } = useRefresh();
  const [ret, setRet] = useState<T>(def);
  const [isLoading, startTransition] = useTransition();
  useEffect(() => {
    startTransition(async () => {
      const result = await fn(arg);
      setRet(result);
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    refreshToken,
    startTransition,
    setRet,
    fn,
    // eslint-disable-next-line react-hooks/exhaustive-deps
    ...Object.entries(arg).flat(),
  ]);
  return { ret, isLoading };
}

// fn is a function that takes one argument and returns a promise
export function makeUseRepo<T, Arg extends object>(
  fn: (args: Arg) => Promise<T>,
  def: T,
) {
  return (arg: Arg) => useRepo<T, Arg>(fn, def, arg);
}

export const useAppSessionUsages = makeUseRepo(getAppSessionUsages, {});
export const useInteractionPeriods = makeUseRepo(getInteractionPeriods, []);
export const useSystemEvents = makeUseRepo(getSystemEvents, []);

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

export function useTagDurationsPerPeriod({
  start,
  end,
  period,
  tag,
}: {
  start?: DateTime;
  end?: DateTime;
  period?: Duration;
  tag?: Tag;
}) {
  const { refreshToken } = useRefresh();
  const [ret, setRet] = useState<{
    tagUsage: number;
    totalUsage: number;
    usages: EntityMap<Tag, WithGroupedDuration<Tag>[]>;
    start?: DateTime;
    end?: DateTime;
    period?: Duration;
  }>({ tagUsage: 0, totalUsage: 0, usages: {}, start, end, period });
  const [isLoading, startTransition] = useTransition();
  useEffect(() => {
    startTransition(async () => {
      if (!start || !end || !period) return;

      const usages = await getTagDurationsPerPeriod({
        start,
        end,
        period,
      });
      const tagUsage = tag?.id ? _(usages[tag.id]).sumBy("duration") : 0;
      const totalUsage = _(usages).values().flatten().sumBy("duration");
      setRet({
        tagUsage,
        totalUsage,
        usages,
        start,
        end,
        period,
      });
    });
  }, [start, end, period, refreshToken, startTransition, tag?.id, tag?.apps]);
  return { ...ret, isLoading };
}

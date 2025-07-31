import { useRefresh } from "@/hooks/use-refresh";
import type { App, Ref, Tag, WithGroupedDuration } from "@/lib/entities";
import {
  getAppDurations,
  getAppDurationsPerPeriod,
  getAppSessionUsages,
  getInteractionPeriods,
  getSystemEvents,
  getTagDurationsPerPeriod,
} from "@/lib/repo";
import type { EntityMap } from "@/lib/state";
import type { Period } from "@/lib/time";
import _ from "lodash";
import type { DateTime } from "luxon";
import { useEffect, useMemo, useRef, useState, useTransition } from "react";
import useSWRImmutable from "swr/immutable";

type RepoFn<Args, Result> = (args: Args) => Promise<Result>;

interface RepoKey<Args, Result> {
  fn: RepoFn<Args, Result>;
  refreshToken: DateTime;
  args: Args;
}

export function useRepo<
  Args,
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  Result extends {},
>(fn: RepoFn<Args, Result>, args: Args, def: Result) {
  const { refreshToken } = useRefresh();
  // ref: https://swr.vercel.app/docs/revalidation#disable-automatic-revalidations
  // SWRImmutable is used to disable automatic revalidations - since we only rely
  // on our own manual refresh using the refreshToken.
  // Object keys are allowed: https://swr.vercel.app/docs/arguments#passing-objects
  // note that they are serialized so it's deep equality
  const swrResult = useSWRImmutable<Result>(
    {
      fn,
      refreshToken,
      args,
    } as RepoKey<Args, Result>,
    // ignore refreshToken
    ({ fn, args }: RepoKey<Args, Result>) => {
      return fn(args);
    },
  );
  const ret = useMemo(() => {
    const { data, ...rest } = swrResult;
    return {
      ...rest,
      ret: data ?? def,
    };
  }, [swrResult, def]);
  return ret;
}

export function makeUseRepo<
  Args,
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  Result extends {},
>(fn: RepoFn<Args, Result>, def: Result) {
  return (arg: Args) => useRepo<Args, Result>(fn, arg, def);
}

export const useAppDurations = makeUseRepo(getAppDurations, {});
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
  period?: Period;
  appId?: Ref<App>;
}) {
  const { refreshToken } = useRefresh();
  const [ret, setRet] = useState<{
    totalAppUsage: number;
    totalUsage: number;
    appDurationsPerPeriod: EntityMap<App, WithGroupedDuration<App>[]>;
    start?: DateTime;
    end?: DateTime;
    period?: Period;
  }>({
    totalAppUsage: 0,
    totalUsage: 0,
    appDurationsPerPeriod: {},
    start,
    end,
    period,
  });
  const [isLoading, startTransition] = useTransition();
  const latestRequestRef = useRef<number>(0);
  const requestCounterRef = useRef<number>(0);

  useEffect(() => {
    startTransition(async () => {
      if (!start || !end || !period) return;

      // Increment the counter for each new request
      const requestId = ++requestCounterRef.current;
      latestRequestRef.current = requestId;

      const appDurationsPerPeriod = await getAppDurationsPerPeriod({
        start,
        end,
        period,
      });

      // Only update state if this is still the latest request
      if (latestRequestRef.current === requestId) {
        const totalAppUsage = appId
          ? _(appDurationsPerPeriod[appId]).sumBy("duration")
          : 0;
        const totalUsage = _(appDurationsPerPeriod)
          .values()
          .flatten()
          .sumBy("duration");
        setRet({
          totalAppUsage,
          totalUsage,
          appDurationsPerPeriod,
          start,
          end,
          period,
        });
      }
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
  period?: Period;
  tag?: Tag;
}) {
  const { refreshToken } = useRefresh();
  const [ret, setRet] = useState<{
    tagUsage: number;
    totalUsage: number;
    usages: EntityMap<Tag, WithGroupedDuration<Tag>[]>;
    start?: DateTime;
    end?: DateTime;
    period?: Period;
  }>({ tagUsage: 0, totalUsage: 0, usages: {}, start, end, period });
  const [isLoading, startTransition] = useTransition();
  const latestRequestRef = useRef<number>(0);
  const requestCounterRef = useRef<number>(0);

  useEffect(() => {
    startTransition(async () => {
      if (!start || !end || !period) return;

      // Increment the counter for each new request
      const requestId = ++requestCounterRef.current;
      latestRequestRef.current = requestId;

      const usages = await getTagDurationsPerPeriod({
        start,
        end,
        period,
      });

      // Only update state if this is still the latest request
      if (latestRequestRef.current === requestId) {
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
      }
    });
  }, [start, end, period, refreshToken, startTransition, tag?.id, tag?.apps]);
  return { ...ret, isLoading };
}

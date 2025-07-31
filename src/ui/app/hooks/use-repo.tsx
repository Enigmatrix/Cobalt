import { useRefresh } from "@/hooks/use-refresh";
import type { Ref, WithGroupedDuration } from "@/lib/entities";
import {
  getAppDurations,
  getAppDurationsPerPeriod,
  getAppSessionUsages,
  getInteractionPeriods,
  getSystemEvents,
  getTagDurationsPerPeriod,
} from "@/lib/repo";
import type { EntityMap } from "@/lib/state";
import { getScore } from "@/lib/stats";
import _ from "lodash";
import type { DateTime } from "luxon";
import { useMemo } from "react";
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
export const useAppDurationsPerPeriod = makeUseRepo(
  getAppDurationsPerPeriod,
  {},
);
export const useTagDurationsPerPeriod = makeUseRepo(
  getTagDurationsPerPeriod,
  {},
);
export const useScore = makeUseRepo(getScore, 0);

export function useTotalUsageFromPerPeriod<T>(
  durationsPerPeriod: EntityMap<T, WithGroupedDuration<T>[]>,
) {
  return useMemo(() => {
    return _(durationsPerPeriod).values().flatten().sumBy("duration");
  }, [durationsPerPeriod]);
}

export function useSingleEntityUsageFromPerPeriod<T>(
  durationsPerPeriod: EntityMap<T, WithGroupedDuration<T>[]>,
  id: Ref<T>,
) {
  return useMemo(() => {
    return _(durationsPerPeriod[id]).sumBy("duration");
  }, [durationsPerPeriod, id]);
}

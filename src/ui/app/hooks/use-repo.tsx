import { useRefresh } from "@/hooks/use-refresh";
import { useConfig } from "@/lib/config";
import type { Ref, Streak, WithGroupedDuration } from "@/lib/entities";
import {
  getAlertEvents,
  getAlertReminderEvents,
  getAppDurations,
  getAppDurationsPerPeriod,
  getAppSessionUsages,
  getInteractionPeriods,
  getSystemEvents,
  getTagDurationsPerPeriod,
} from "@/lib/repo";
import type { EntityMap } from "@/lib/state";
import { getScore, getScorePerPeriod, getStreaks } from "@/lib/stats";
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
    // on app refresh, this doesn't reset the data to the default
    // value (so prevents a lot of screen flashes)
    { keepPreviousData: true },
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
export const useAlertEvents = makeUseRepo(getAlertEvents, []);
export const useAlertReminderEvents = makeUseRepo(getAlertReminderEvents, []);
export const useAppDurationsPerPeriod = makeUseRepo(
  getAppDurationsPerPeriod,
  {},
);
export const useTagDurationsPerPeriod = makeUseRepo(
  getTagDurationsPerPeriod,
  {},
);
export const useScore = makeUseRepo(getScore, 0);
export const useScorePerPeriod = makeUseRepo(getScorePerPeriod, []);
export const useStreaks = makeUseRepo(getStreaks, []);
export const useDefaultStreaks = function (arg: {
  start: DateTime;
  end: DateTime;
}) {
  const { defaultFocusStreakSettings, defaultDistractiveStreakSettings } =
    useConfig();
  return useStreaks({
    ...arg,
    focusSettings: defaultFocusStreakSettings,
    distractiveSettings: defaultDistractiveStreakSettings,
  });
};

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

export function useStreakDurations(streaks?: Streak[]) {
  return useMemo(() => {
    if (!streaks) {
      return [0, 0];
    }
    const focusStreakUsage = _(streaks)
      .filter((streak) => streak.isFocused)
      .sumBy((streak) => streak.end - streak.start);
    const distractiveStreakUsage = _(streaks)
      .filter((streak) => !streak.isFocused)
      .sumBy((streak) => streak.end - streak.start);
    return [focusStreakUsage, distractiveStreakUsage];
  }, [streaks]);
}

import type { Score, WithGroup } from "@/lib/entities";
import { getQueryOptions, type QueryOptions } from "@/lib/repo";
import { dateTimeToTicks, type Period } from "@/lib/time";
import { invoke } from "@tauri-apps/api/core";
import { DateTime } from "luxon";

export async function getScore({
  options,
  start,
  end,
}: {
  options?: QueryOptions;
  start: DateTime;
  end: DateTime;
}): Promise<Score> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_score", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
  });
}

export async function getScorePerPeriod({
  options,
  start,
  end,
  period,
}: {
  options?: QueryOptions;
  start: DateTime;
  end: DateTime;
  period: Period;
}): Promise<WithGroup<Score>[]> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_score_per_period", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
    period,
  });
}

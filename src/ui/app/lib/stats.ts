import { getQueryOptions, type QueryOptions } from "@/lib/repo";
import { dateTimeToTicks } from "@/lib/time";
import { invoke } from "@tauri-apps/api/core";
import { DateTime } from "luxon";

export const SCORE_SENTINEL = null as unknown as number;

export async function getScore({
  options,
  start,
  end,
}: {
  options?: QueryOptions;
  start: DateTime;
  end: DateTime;
}): Promise<number> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_score", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
  });
}

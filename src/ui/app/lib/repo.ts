import type {
  Timestamp,
  WithDuration,
  WithGroupedDuration,
  App,
  Tag,
} from "@/lib/entities";
import { invoke } from "@tauri-apps/api/core";
import type { EntityMap, EntityStore } from "./state";
import type { DateTime, Duration } from "luxon";
import { dateTimeToTicks, durationToTicks } from "./time";

interface QueryOptions {
  now?: Timestamp;
}

function getQueryOptions(queryOptions?: QueryOptions): QueryOptions {
  return queryOptions || {};
}

export async function getApps({
  options,
}: {
  options?: QueryOptions;
}): Promise<EntityStore<App>> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_apps", { queryOptions });
}

export async function getTags({
  options,
}: {
  options?: QueryOptions;
}): Promise<EntityStore<Tag>> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_tags", { queryOptions });
}

export async function getAppDurations({
  options,
  start,
  end,
}: {
  options?: QueryOptions;
  start: DateTime;
  end: DateTime;
}): Promise<EntityMap<App, WithDuration<App>>> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_app_durations", {
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
    queryOptions,
  });
}

export async function getAppDurationsPerPeriod({
  options,
  start,
  end,
  period,
}: {
  options?: QueryOptions;
  start: DateTime;
  end: DateTime;
  period: Duration;
}): Promise<EntityMap<App, WithGroupedDuration<App>[]>> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_app_durations_per_period", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
    period: durationToTicks(period),
  });
}

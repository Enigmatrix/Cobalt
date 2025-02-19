import type {
  Timestamp,
  WithDuration,
  WithGroupedDuration,
  App,
  Session,
  Tag,
  Ref,
} from "@/lib/entities";
import { invoke } from "@tauri-apps/api/core";
import type { EntityMap, EntityStore } from "@/lib/state";
import type { DateTime, Duration } from "luxon";
import { dateTimeToTicks, durationToTicks } from "@/lib/time";

export interface CreateTag {
  name: string;
  color: string;
}

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

export async function getTagDurationsPerPeriod({
  options,
  start,
  end,
  period,
}: {
  options?: QueryOptions;
  start: DateTime;
  end: DateTime;
  period: Duration;
}): Promise<EntityMap<Tag, WithGroupedDuration<Tag>[]>> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_tag_durations_per_period", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
    period: durationToTicks(period),
  });
}

export async function updateApp(app: App): Promise<void> {
  const updatedApp = {
    id: app.id,
    name: app.name,
    description: app.description,
    company: app.company,
    color: app.color,
    tag_id: app.tag_id,
  };
  return await invoke("update_app", { app: updatedApp });
}

export async function updateTag(tag: Tag): Promise<void> {
  const updatedTag = {
    id: tag.id,
    name: tag.name,
    color: tag.color,
  };
  return await invoke("update_tag", { tag: updatedTag });
}

export async function updateTagApps(
  tagId: Ref<Tag>,
  removedApps: Ref<App>[],
  addedApps: Ref<App>[],
): Promise<void> {
  return await invoke("update_tag_apps", { tagId, removedApps, addedApps });
}

export async function createTag(tag: CreateTag): Promise<Ref<Tag>> {
  return await invoke("create_tag", { tag });
}

export async function removeTag(tagId: Ref<Tag>): Promise<void> {
  return await invoke("remove_tag", { tagId });
}

export type AppSessionUsages = {
  [appId: Ref<App>]: {
    [sessionId: Ref<Session>]: Session;
  };
};

export async function appSessionUsages({
  options,
  start,
  end,
}: {
  options?: QueryOptions;
  start: DateTime;
  end: DateTime;
}): Promise<AppSessionUsages> {
  const queryOptions = getQueryOptions(options);
  return await invoke("app_session_usages", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
  });
}

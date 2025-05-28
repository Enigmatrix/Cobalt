import type {
  Timestamp,
  WithDuration,
  WithGroupedDuration,
  Duration as DataDuration,
  App,
  Session,
  Tag,
  Ref,
  InteractionPeriod,
  SystemEvent,
  Alert,
  Target,
  TimeFrame,
  TriggerAction,
  Reminder,
} from "@/lib/entities";
import { invoke } from "@tauri-apps/api/core";
import type { EntityMap, EntityStore } from "@/lib/state";
import { DateTime } from "luxon";
import { dateTimeToTicks, type Period } from "@/lib/time";

export interface CreateTag {
  name: string;
  color: string;
  score: number;
  apps: Ref<App>[];
}

export type AppSessionUsages = {
  [appId: Ref<App>]: {
    [sessionId: Ref<Session>]: Session;
  };
};

export interface CreateAlert {
  target: Target;
  usageLimit: DataDuration;
  timeFrame: TimeFrame;
  triggerAction: TriggerAction;
  reminders: CreateReminder[];
  ignoreTrigger: boolean;
}

export interface CreateReminder {
  threshold: number;
  message: string;
  ignoreTrigger: boolean;
}

export interface UpdatedAlert {
  id: Ref<Alert>;
  target: Target;
  usageLimit: DataDuration;
  timeFrame: TimeFrame;
  triggerAction: TriggerAction;
  reminders: UpdatedReminder[];
  ignoreTrigger: boolean;
}

export interface UpdatedReminder {
  // undefined when creating a new reminder
  id?: Ref<Reminder>;
  threshold: number;
  message: string;
  ignoreTrigger: boolean;
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

export async function getAlerts({
  options,
}: {
  options?: QueryOptions;
}): Promise<EntityStore<Alert>> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_alerts", { queryOptions });
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
  period: Period;
}): Promise<EntityMap<App, WithGroupedDuration<App>[]>> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_app_durations_per_period", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
    period,
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
  period: Period;
}): Promise<EntityMap<Tag, WithGroupedDuration<Tag>[]>> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_tag_durations_per_period", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
    period,
  });
}

export async function updateApp(app: App): Promise<void> {
  const updatedApp = {
    id: app.id,
    name: app.name,
    description: app.description,
    company: app.company,
    color: app.color,
    tagId: app.tagId,
  };
  return await invoke("update_app", { app: updatedApp });
}

export async function updateTag(tag: Tag): Promise<void> {
  const updatedTag = {
    id: tag.id,
    name: tag.name,
    color: tag.color,
    score: tag.score,
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

export async function createTag(tag: CreateTag): Promise<Tag> {
  return await invoke("create_tag", { tag });
}

export async function removeTag(tagId: Ref<Tag>): Promise<void> {
  return await invoke("remove_tag", { tagId });
}

export async function createAlert(alert: CreateAlert): Promise<Alert> {
  return await invoke("create_alert", { alert });
}

export async function updateAlert(
  prev: Alert,
  next: UpdatedAlert,
): Promise<Alert> {
  return await invoke("update_alert", { prev, next });
}

export async function removeAlert(alertId: Ref<Alert>): Promise<void> {
  return await invoke("remove_alert", { alertId });
}

export async function createAlertEventIgnore(
  alertId: Ref<Alert>,
): Promise<void> {
  const timestamp = dateTimeToTicks(DateTime.now());
  return await invoke("create_alert_event_ignore", { alertId, timestamp });
}

export async function getAppSessionUsages({
  options,
  start,
  end,
}: {
  options?: QueryOptions;
  start: DateTime;
  end: DateTime;
}): Promise<AppSessionUsages> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_app_session_usages", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
  });
}

export async function getInteractionPeriods({
  options,
  start,
  end,
}: {
  options?: QueryOptions;
  start: DateTime;
  end: DateTime;
}): Promise<InteractionPeriod[]> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_interaction_periods", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
  });
}

export async function getSystemEvents({
  options,
  start,
  end,
}: {
  options?: QueryOptions;
  start: DateTime;
  end: DateTime;
}): Promise<SystemEvent[]> {
  const queryOptions = getQueryOptions(options);
  return await invoke("get_system_events", {
    queryOptions,
    start: dateTimeToTicks(start),
    end: dateTimeToTicks(end),
  });
}

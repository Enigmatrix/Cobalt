export type Ref<T> = number & { __type: T };
export function newRef<T>(id: number): Ref<T> {
  return id as Ref<T>;
}
export type Color = string;
export type Timestamp = number;
export type Duration = number;

export interface WithDuration<T> {
  id: Ref<T>;
  duration: Duration;
}

export interface WithGroupedDuration<T> {
  id: Ref<T>;
  duration: Duration;
  group: Timestamp;
}

export type AppIdentity =
  | { Uwp: { aumid: string } }
  | { Win32: { path: string } };

export function isUwp(
  identity: AppIdentity,
): identity is { Uwp: { aumid: string } } {
  return "Uwp" in identity;
}

export function isWin32(
  identity: AppIdentity,
): identity is { Win32: { path: string } } {
  return "Win32" in identity;
}

export interface ValuePerPeriod<T> {
  today: T;
  week: T;
  month: T;
}

export interface App {
  id: Ref<App>;
  name: string;
  description: string;
  company: string;
  color: Color;
  identity: AppIdentity;
  icon: Buffer;
  tag_id: Ref<Tag> | null;
  usages: ValuePerPeriod<Duration>;
}

export interface Session {
  id: Ref<Session>;
  title: string;
  start: Timestamp;
  end: Timestamp;
  usages: Usage[];
}

export interface Usage {
  id: Ref<Usage>;
  start: Timestamp;
  end: Timestamp;
}

export interface Tag {
  id: Ref<Tag>;
  name: string;
  color: string;
  apps: Ref<App>[];
  usages: ValuePerPeriod<Duration>;
}

export type Target = { App: Ref<App> } | { Tag: Ref<Tag> };

export type TimeFrame = "Daily" | "Weekly" | "Monthly";

export type TriggerAction =
  | { Kill: null }
  | { Dim: number }
  | { Message: string };

export interface Alert {
  id: Ref<Alert>;
  target: Target;
  usage_limit: Duration;
  time_frame: TimeFrame;
  trigger_action: TriggerAction;

  reminders: Reminder[];
  events: ValuePerPeriod<number>;
}

export interface Reminder {
  id: Ref<Reminder>;
  alert_id: Ref<Alert>;
  threshold: number;
  message: string;

  events: ValuePerPeriod<number>;
}

export interface InteractionPeriod {
  id: Ref<Usage>;
  start: Timestamp;
  end: Timestamp;
  mouse_clicks: number;
  key_strokes: number;
}

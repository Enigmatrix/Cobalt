export type Ref<T> = number & { __type: T };
export function newRef<T>(id: number): Ref<T> {
  return id as Ref<T>;
}
export type Color = string;
export type Timestamp = number;
export type Duration = number;
export type Period = "hour" | "day" | "week" | "month" | "year";
export const PERIODS: Period[] = [
  "hour",
  "day",
  "week",
  "month",
  "year",
] as const;

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
  | { tag: "Uwp"; aumid: string }
  | { tag: "Win32"; path: string };

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

export type Target =
  | { tag: "App"; id: Ref<App> }
  | { tag: "Tag"; id: Ref<Tag> };

export type TimeFrame = "Daily" | "Weekly" | "Monthly";

export function timeFrameToPeriod(timeFrame: TimeFrame): Period {
  switch (timeFrame) {
    case "Daily":
      return "day";
    case "Weekly":
      return "week";
    case "Monthly":
      return "month";
  }
}

export type TriggerAction =
  | { tag: "Kill" }
  | { tag: "Dim"; duration: number }
  | { tag: "Message"; content: string };

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

export enum SystemEventEnum {
  Shutdown = 0,
  Logoff = 1,
  Lock = 2,
  Unlock = 3,
  Suspend = 4,
  Resume = 5,
  MonitorOn = 6,
  MonitorOff = 7,
}

export function systemEventToString(event: SystemEventEnum) {
  switch (event) {
    case SystemEventEnum.Shutdown:
      return "Shutdown";
    case SystemEventEnum.Logoff:
      return "Logoff";
    case SystemEventEnum.Lock:
      return "Lock";
    case SystemEventEnum.Unlock:
      return "Unlock";
    case SystemEventEnum.Suspend:
      return "Suspend";
    case SystemEventEnum.Resume:
      return "Resume";
    case SystemEventEnum.MonitorOn:
      return "Monitor On";
    case SystemEventEnum.MonitorOff:
      return "Monitor Off";
    default:
      throw new Error(`Unknown system event: ${event}`);
  }
}

export interface SystemEvent {
  id: Ref<SystemEvent>;
  timestamp: Timestamp;
  event: SystemEventEnum;
}

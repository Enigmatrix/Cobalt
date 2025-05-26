export type Ref<T> = number & { __type: T };
export function newRef<T>(id: number): Ref<T> {
  return id as Ref<T>;
}
export type Color = string;
export type Timestamp = number;
export type Duration = number;
type Period = "hour" | "day" | "week" | "month" | "year";
// change the exported type so that users use the time.ts Period type instead
export type EntityPeriod = Period;

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
  | { tag: "uwp"; aumid: string }
  | { tag: "win32"; path: string }
  | { tag: "website"; baseUrl: string };

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
  tagId: Ref<Tag> | null;
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
  | { tag: "app"; id: Ref<App> }
  | { tag: "tag"; id: Ref<Tag> };

export type TimeFrame = "daily" | "weekly" | "monthly";

export function timeFrameToPeriod(timeFrame: TimeFrame): Period {
  switch (timeFrame) {
    case "daily":
      return "day";
    case "weekly":
      return "week";
    case "monthly":
      return "month";
  }
}

export type TriggerAction =
  | { tag: "kill" }
  | { tag: "dim"; duration: number }
  | { tag: "message"; content: string };

export type AlertTriggerStatus =
  | { tag: "untriggered" }
  | { tag: "hit"; timestamp: Timestamp }
  | { tag: "ignored"; timestamp: Timestamp };

export type ReminderTriggerStatus =
  | { tag: "untriggered" }
  | { tag: "hit"; timestamp: Timestamp }
  | { tag: "ignored"; timestamp: Timestamp; ignoredByAlert: boolean };

export interface Alert {
  id: Ref<Alert>;
  target: Target;
  usageLimit: Duration;
  timeFrame: TimeFrame;
  triggerAction: TriggerAction;
  reminders: Reminder[];

  status: AlertTriggerStatus;
  events: ValuePerPeriod<number>;
}

export interface Reminder {
  id: Ref<Reminder>;
  alertId: Ref<Alert>;
  threshold: number;
  message: string;

  status: ReminderTriggerStatus;
  events: ValuePerPeriod<number>;
}

export interface InteractionPeriod {
  id: Ref<Usage>;
  start: Timestamp;
  end: Timestamp;
  mouseClicks: number;
  keyStrokes: number;
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

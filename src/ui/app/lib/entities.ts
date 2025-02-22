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

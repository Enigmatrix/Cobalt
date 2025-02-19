export type Ref<T> = number & { __type: T };

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

export interface UsageInfo {
  usage_today: number;
  usage_week: number;
  usage_month: number;
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
  usages: UsageInfo;
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
  usages: UsageInfo;
}

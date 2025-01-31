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
  tags: Ref<Tag>[];
  usages: UsageInfo;
}

export interface Tag {
  id: Ref<Tag>;
  name: string;
  color: string;
  apps: Ref<App>[];
  usages: UsageInfo;
}

export type Ref<T> = number & { __type: T };

export type Color = string;
export type Timestamp = number;
export type Duration = number;

export type AppIdentity =
  | { Uwp: { aumid: string } }
  | { Win32: { path: string } };

export interface App {
  id: Ref<App>;
  name: string;
  description: string;
  company: string;
  color: Color;
  identity: AppIdentity;
  icon: Buffer;
  tags: Ref<Tag>[];
}

export interface Tag {
  id: Ref<Tag>;
  name: string;
  color: string;
  apps: Ref<App>[];
}

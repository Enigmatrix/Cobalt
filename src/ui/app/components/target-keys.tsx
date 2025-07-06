import type { App, Ref, Tag } from "@/lib/entities";
import type { EntityStore } from "@/lib/state";
import { untagged, untaggedIdMarker } from "@/lib/state";

export interface AppFullKey {
  key: "app";
  app: App;
}
export interface TagFullKey {
  key: "tag";
  tag: Tag;
}
export type FullKey = AppFullKey | TagFullKey;

export interface AppKey {
  key: "app";
  id: Ref<App>;
}
export interface TagKey {
  key: "tag";
  id: Ref<Tag>;
}
export type Key = AppKey | TagKey;

export function keyToString(key: Key) {
  return key.key === "app" ? "app-" + key.id : "tag-" + key.id;
}

export function fullKeyToString(key: FullKey) {
  return key.key === "app" ? "app-" + key.app.id : "tag-" + key.tag.id;
}

export function stringToKey(str: string): Key {
  const [key, idStr] = str.split("-", 2);
  if (key === "app") {
    return { key: "app", id: +idStr as Ref<App> };
  } else if (key === "tag") {
    if (str === untaggedIdMarker) {
      return { key: "tag", id: untagged.id };
    } else {
      return { key: "tag", id: +idStr as Ref<Tag> };
    }
  }
  throw new Error("Invalid key: " + str);
}

export function stringToFullKey(
  str: string,
  apps: EntityStore<App>,
  tags: EntityStore<Tag>,
): FullKey {
  const [key, idStr] = str.split("-", 2);
  if (key === "app") {
    return { key: "app", app: apps[+idStr as Ref<App>]! };
  } else if (key === "tag") {
    if (str === untaggedIdMarker) {
      return { key: "tag", tag: untagged };
    } else {
      return { key: "tag", tag: tags[+idStr as Ref<Tag>]! };
    }
  }
  throw new Error("Invalid key: " + str);
}

export type FullKeyWith<T> = FullKey & T;
export type FullKeyWithDuration = FullKeyWith<{ duration: number }>;
export type KeyWith<T> = Key & T;
export type KeyWithDuration = KeyWith<{ duration: number }>;

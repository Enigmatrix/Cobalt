import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { App, Ref, Tag } from "@/lib/entities";
import { DateTime } from "luxon";
import { dateTimeToTicks } from "@/lib/time";
import {
  createTag,
  getApps,
  getTags,
  removeTag,
  updateApp,
  updateTag,
  type CreateTag,
} from "@/lib/repo";
import { checkForUpdatesBackground } from "@/lib/updater";
import { info } from "@/lib/log";
import { produce } from "immer";

export async function initState() {
  // init rust-side state
  await invoke("init_state");
  await refresh();
  if (import.meta.env.PROD) {
    checkForUpdatesBackground();
  }
}

export async function refresh() {
  const now = DateTime.now();
  const options = { now: dateTimeToTicks(now) };
  const [apps, tags] = await Promise.all([
    getApps({ options }),
    getTags({ options }),
  ]);
  useAppState.setState({ apps, tags, lastRefresh: now });
  info("refresh completed");
}

// Entity store with T|undefined values. This is because
// the store can be stale.
export type EntityStore<T> = Record<Ref<T>, T | undefined>;
export type EntityMap<T, V> = Record<Ref<T>, V | undefined>;

type AppState = {
  lastRefresh: DateTime;
  apps: EntityStore<App>;
  tags: EntityStore<Tag>;
  updateApp: (app: App) => Promise<void>;
  updateTag: (app: Tag) => Promise<void>;
  createTag: (tag: CreateTag) => Promise<Ref<Tag>>;
  removeTag: (tagId: Ref<Tag>) => Promise<void>;
};

export const useAppState = create<AppState>((set) => {
  return {
    lastRefresh: DateTime.now(),
    apps: [],
    tags: [],
    updateApp: async (app) => {
      await updateApp(app);

      set((state) =>
        produce((draft: AppState) => {
          draft.apps[app.id] = { ...draft.apps[app.id], ...app };
        })(state),
      );
    },
    updateTag: async (tag) => {
      await updateTag(tag);

      set((state) =>
        produce((draft: AppState) => {
          draft.tags[tag.id] = { ...draft.tags[tag.id], ...tag };
        })(state),
      );
    },
    createTag: async (tag) => {
      const tagId = await createTag(tag);
      set((state) =>
        produce((draft: AppState) => {
          draft.tags[tagId] = {
            id: tagId,
            ...tag,
            apps: [],
            usages: {
              usage_today: 0,
              usage_week: 0,
              usage_month: 0,
            },
          };
        })(state),
      );

      return tagId;
    },
    removeTag: async (tagId) => {
      await removeTag(tagId);
      set((state) =>
        produce((draft: AppState) => {
          delete draft.tags[tagId];
        })(state),
      );
    },
  };
});

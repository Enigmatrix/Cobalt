import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { App, Ref, Tag } from "@/lib/entities";
import { DateTime } from "luxon";
import { dateTimeToTicks } from "@/lib/time";
import {
  addAppTag,
  getApps,
  getTags,
  removeAppTag,
  updateApp,
} from "@/lib/repo";
import { checkForUpdatesBackground } from "@/lib/updater";
import { info } from "@/lib/log";
import { produce } from "immer";

export async function initState() {
  // init rust-side state
  await invoke("init_state");
  info("rust-side state initialized");
  await refresh();
  if (import.meta.env.PROD) {
    checkForUpdatesBackground();
  }
}

export async function refresh() {
  const state = useAppState.getState();
  const now = DateTime.now();
  const options = { now: dateTimeToTicks(now) };
  const [apps, tags] = await Promise.all([
    getApps({ options }),
    getTags({ options }),
  ]);
  state.setApps(apps as Record<Ref<App>, App>);
  state.setTags(tags as Record<Ref<Tag>, Tag>);
  state.setLastRefresh(now);
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
  setApps: (apps: EntityStore<App>) => void;
  setTags: (tags: EntityStore<Tag>) => void;
  setLastRefresh: (lastRefresh: DateTime) => void;
  updateApp: (app: App) => Promise<void>;
  addAppTag: (appId: Ref<App>, tagId: Ref<Tag>) => Promise<void>;
  removeAppTag: (appId: Ref<App>, tagId: Ref<Tag>) => Promise<void>;
};

export const useAppState = create<AppState>((set) => {
  return {
    lastRefresh: DateTime.now(),
    apps: [],
    tags: [],
    setApps: (apps) => set({ apps }),
    setTags: (tags) => set({ tags }),
    setLastRefresh: (lastRefresh) => set({ lastRefresh }),

    updateApp: async (app) => {
      await updateApp(app);

      set((state) =>
        produce(
          (draft) => (draft.apps[app.id] = { ...draft.apps[app.id], ...app }),
        )(state),
      );
    },

    addAppTag: async (appId, tagId) => {
      await addAppTag(appId, tagId);

      set((state) =>
        produce((draft: AppState) => {
          if (draft.apps[appId]!.tags.includes(tagId)) {
            return;
          }
          draft.apps[appId]!.tags.push(tagId);
        })(state),
      );
    },

    removeAppTag: async (appId, tagId) => {
      await removeAppTag(appId, tagId);

      set((state) =>
        produce((draft: AppState) => {
          draft.apps[appId]!.tags = draft.apps[appId]!.tags.filter(
            (id: Ref<Tag>) => id !== tagId,
          );
        })(state),
      );
    },
  };
});

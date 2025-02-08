import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { App, Ref, Tag } from "@/lib/entities";
import { DateTime } from "luxon";
import { dateTimeToTicks } from "@/lib/time";
import { getApps, getTags } from "@/lib/repo";
import { checkForUpdatesBackground } from "@/lib/updater";
import { info } from "@/lib/log";

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
};

export const useAppState = create<AppState>((set) => {
  return {
    lastRefresh: DateTime.now(),
    apps: [],
    tags: [],
    setApps: (apps) => set({ apps }),
    setTags: (tags) => set({ tags }),
    setLastRefresh: (lastRefresh) => set({ lastRefresh }),
  };
});

import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { App, Ref, Tag } from "./entities";
import { DateTime } from "luxon";
import { toTicks } from "./time";

export async function initState() {
  // init rust-side state
  await invoke("init_state");
  await refresh();
}

export async function refresh() {
  const state = useAppState.getState();
  const now = DateTime.now();
  const [apps, tags] = await Promise.all([
    await invoke("get_apps", { queryOptions: { now: toTicks(now) } }),
    await invoke("get_tags", { queryOptions: { now: toTicks(now) } }),
  ]);
  state.setApps(apps as Record<Ref<App>, App>);
  state.setTags(tags as Record<Ref<Tag>, Tag>);
}

type AppState = {
  apps: Record<Ref<App>, App>;
  tags: Record<Ref<Tag>, Tag>;
  setApps: (apps: Record<Ref<App>, App>) => void;
  setTags: (tags: Record<Ref<Tag>, Tag>) => void;
};

export const useAppState = create<AppState>((set) => {
  return {
    apps: [],
    tags: [],
    setApps: (apps) => set({ apps }),
    setTags: (tags) => set({ tags }),
  };
});

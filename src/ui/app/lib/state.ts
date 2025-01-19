import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { App, Ref, Tag } from "./entities";

export async function initState() {
  // init rust-side state
  await invoke("init_state");

  const state = useAppState.getState();
  const [apps, tags] = await Promise.all([
    await invoke("get_apps"),
    await invoke("get_tags"),
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

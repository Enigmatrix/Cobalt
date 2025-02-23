import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { Alert, App, Ref, Tag } from "@/lib/entities";
import { DateTime } from "luxon";
import { dateTimeToTicks } from "@/lib/time";
import {
  createAlert,
  createTag,
  getAlerts,
  getApps,
  getTags,
  removeAlert,
  removeTag,
  updateAlert,
  updateApp,
  updateTag,
  updateTagApps,
  type CreateAlert,
  type CreateTag,
  type UpdatedAlert,
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
  const [apps, tags, alerts] = await Promise.all([
    getApps({ options }),
    getTags({ options }),
    getAlerts({ options }),
  ]);
  useAppState.setState({ apps, tags, alerts, lastRefresh: now });
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
  alerts: EntityStore<Alert>;
  updateApp: (app: App) => Promise<void>;
  updateTag: (app: Tag) => Promise<void>;
  updateTagApps: (tag: Tag, apps: Ref<App>[]) => Promise<void>;
  createTag: (tag: CreateTag) => Promise<Ref<Tag>>;
  removeTag: (tagId: Ref<Tag>) => Promise<void>;
  createAlert: (tag: CreateAlert) => Promise<Ref<Alert>>;
  updateAlert: (prev: Alert, next: UpdatedAlert) => Promise<void>;
  removeAlert: (tagId: Ref<Alert>) => Promise<void>;
};

export const useAppState = create<AppState>((set) => {
  return {
    lastRefresh: DateTime.now(),
    apps: [],
    tags: [],
    alerts: [],
    updateApp: async (app) => {
      await updateApp(app);

      set((state) =>
        produce((draft: AppState) => {
          const oldTagId = draft.apps[app.id]?.tag_id;
          const newTagId = app.tag_id;
          if (oldTagId) {
            const apps = draft.tags[oldTagId]?.apps;
            if (apps) {
              apps.splice(apps.indexOf(app.id), 1);
            }
          }
          if (newTagId) {
            const apps = draft.tags[newTagId]?.apps;
            if (apps) {
              apps.push(app.id);
            }
          }
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
    updateTagApps: async (tag, apps) => {
      const removedApps = tag.apps.filter((id) => !apps.includes(id));
      const addedApps = apps.filter((id) => !tag.apps.includes(id));
      await updateTagApps(tag.id, removedApps, addedApps);

      set((state) =>
        produce((draft: AppState) => {
          removedApps.forEach((appId) => {
            draft.apps[appId]!.tag_id = null;
          });

          addedApps.forEach((appId) => {
            draft.apps[appId]!.tag_id = tag.id;
          });

          draft.tags[tag.id]!.apps = apps;
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
              today: 0,
              week: 0,
              month: 0,
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
    createAlert: async (alert) => {
      const newAlert = await createAlert(alert);
      set((state) =>
        produce((draft: AppState) => {
          draft.alerts[newAlert.id] = newAlert;
        })(state),
      );
      return newAlert.id;
    },
    updateAlert: async (prev, next) => {
      const newAlert = await updateAlert(prev, next);
      set((state) =>
        produce((draft: AppState) => {
          delete draft.alerts[prev.id];
          draft.alerts[newAlert.id] = newAlert;
        })(state),
      );
    },
    removeAlert: async (alertId) => {
      await removeAlert(alertId);
      set((state) =>
        produce((draft: AppState) => {
          delete draft.alerts[alertId];
        })(state),
      );
    },
  };
});

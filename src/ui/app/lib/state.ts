import { getTheme } from "@/components/theme-provider";
import { getIconsDir, refresh as refreshConfig } from "@/lib/config";
import type { Alert, App, Ref, Tag } from "@/lib/entities";
import { info } from "@/lib/log";
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
import { dateTimeToTicks } from "@/lib/time";
import { checkForUpdatesBackground } from "@/lib/updater";
import { setTheme } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { produce } from "immer";
import _ from "lodash";
import { DateTime } from "luxon";
import { create } from "zustand";

export async function initState() {
  const theme = getTheme();

  await Promise.all([
    setTheme(theme === "system" ? null : theme),
    invoke("init_state"), // init rust-side state
  ]);

  const [iconsDirOut] = await Promise.all([getIconsDir(), refresh()]);
  iconsDir = iconsDirOut;

  if (import.meta.env.PROD) {
    checkForUpdatesBackground();
  }
}

export let iconsDir: string;

export async function refresh() {
  const now = DateTime.now();
  const options = { now: dateTimeToTicks(now) };
  const [apps, tags, alerts] = await Promise.all([
    getApps({ options }),
    getTags({ options }),
    getAlerts({ options }),
    refreshConfig(),
  ]);
  useAppState.setState({ apps, tags, alerts, lastRefresh: now });
  info("refresh completed");
}

export const untagged: Tag = {
  id: -1 as unknown as Ref<Tag>,
  name: "Untagged",
  color: "#6B7280", // gray-500
  score: 0,
  // below two fields are not really used.
  apps: [],
  usages: {
    today: 0,
    week: 0,
    month: 0,
  },
};

export const untaggedIdMarker = "tag-" + untagged.id;

// Entity store with T|undefined values. This is because
// the store can be stale.
export type EntityStore<T> = Record<Ref<T>, T | undefined>;
export type EntityMap<T, V> = Record<Ref<T>, V | undefined>;

interface AppState {
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
  updateAlert: (prev: Alert, next: UpdatedAlert) => Promise<Ref<Alert>>;
  removeAlert: (tagId: Ref<Alert>) => Promise<void>;
}

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
          const oldTagId = draft.apps[app.id]?.tagId;
          const newTagId = app.tagId;
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
            draft.apps[appId]!.tagId = null;
          });

          addedApps.forEach((appId) => {
            const oldTagId = draft.apps[appId]!.tagId;
            draft.apps[appId]!.tagId = tag.id;
            if (oldTagId) {
              const tag = draft.tags[oldTagId]!;

              // remove app from previous tag's app list
              tag.apps.splice(tag.apps.indexOf(appId), 1);
            }
          });

          draft.tags[tag.id]!.apps = apps;
        })(state),
      );
    },
    createTag: async (tag) => {
      const newTag = await createTag(tag);
      set((state) =>
        produce((draft: AppState) => {
          draft.tags[newTag.id] = newTag;
          newTag.apps.forEach((appId) => {
            const oldTagId = draft.apps[appId]!.tagId;
            draft.apps[appId]!.tagId = newTag.id;
            if (oldTagId) {
              const tag = draft.tags[oldTagId]!;

              // remove app from previous tag's app list
              tag.apps.splice(tag.apps.indexOf(appId), 1);
            }
          });
        })(state),
      );
      return newTag.id;
    },
    removeTag: async (tagId) => {
      await removeTag(tagId);
      set((state) =>
        produce((draft: AppState) => {
          // reset apps using this tag
          draft.tags[tagId]?.apps.forEach((appId) => {
            draft.apps[appId]!.tagId = null;
          });

          // remove alerts using this tag
          draft.alerts = _.omitBy(
            draft.alerts,
            (alert) =>
              alert!.target.tag === "tag" && alert!.target.id === tagId,
          );

          // remove tag
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
      return newAlert.id;
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

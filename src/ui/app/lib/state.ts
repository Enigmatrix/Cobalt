import { getTheme } from "@/components/theme-provider";
import { getIconsDir, refresh as refreshConfig } from "@/lib/config";
import type { Alert, App, Ref, Tag } from "@/lib/entities";
import { info } from "@/lib/log";
import {
  createAlert,
  createAlertEvent,
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
import { mutate } from "swr";
import { create } from "zustand";

const REFRESH_GAP_MS = 60_000;

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

  createRefreshTimer(REFRESH_GAP_MS);
}

export let iconsDir: string;

export let refreshTimer: ReturnType<typeof setInterval> | null = null;

export function createRefreshTimer(gapMs: number) {
  if (refreshTimer) {
    clearInterval(refreshTimer);
  }

  refreshTimer = setInterval(() => {
    void refresh();
  }, gapMs);
}

export async function refresh() {
  const now = DateTime.now();

  // Refresh every SWR cache
  await mutate(() => true);

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

async function refreshParts(
  now: DateTime,
  set: (state: Partial<AppState>) => void,
  {
    refreshApps = false,
    refreshTags = false,
    refreshAlerts = false,
  }: {
    refreshApps?: boolean;
    refreshTags?: boolean;
    refreshAlerts?: boolean;
  } = {},
) {
  const options = { now: dateTimeToTicks(now) };
  const [apps, tags, alerts] = await Promise.all([
    refreshApps ? getApps({ options }) : Promise.resolve(undefined),
    refreshTags ? getTags({ options }) : Promise.resolve(undefined),
    refreshAlerts ? getAlerts({ options }) : Promise.resolve(undefined),
  ]);
  const obj: Partial<AppState> = { lastRefresh: now };
  if (refreshApps) obj.apps = apps;
  if (refreshTags) obj.tags = tags;
  if (refreshAlerts) obj.alerts = alerts;
  set(obj);

  // Refresh every SWR cache
  await mutate(() => true);
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
  toggleAlert: (alert: Alert) => Promise<void>;
}

export const useAppState = create<AppState>((set) => {
  return {
    lastRefresh: DateTime.now(),
    apps: [],
    tags: [],
    alerts: [],
    updateApp: async (app) => {
      await updateApp(app);

      // just refresh ... we need update apps' tags
      // and then durations etc, very annoying
      await refreshParts(DateTime.now(), set, {
        refreshApps: true,
        refreshTags: true,
      });
    },
    updateTag: async (tag) => {
      await updateTag(tag);

      set((state) =>
        produce((draft: AppState) => {
          draft.tags[tag.id] = { ...draft.tags[tag.id], ...tag };
        })(state),
      );
    },
    updateTagApps: async (tag, newApps) => {
      const removedApps = tag.apps.filter((id) => !newApps.includes(id));
      const addedApps = newApps.filter((id) => !tag.apps.includes(id));
      await updateTagApps(tag.id, removedApps, addedApps);

      // just refresh ... we need update apps' tags
      // and then durations etc, very annoying
      await refreshParts(DateTime.now(), set, {
        refreshApps: true,
        refreshTags: true,
      });
    },
    createTag: async (tag) => {
      const newTag = await createTag(tag);
      // just refresh ... we need update apps' tags
      // and then durations etc, very annoying
      await refreshParts(DateTime.now(), set, {
        refreshApps: true,
        refreshTags: true,
      });
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
      await refreshParts(DateTime.now(), set, { refreshAlerts: true });
      return newAlert;
    },
    updateAlert: async (prev, next) => {
      const newAlert = await updateAlert(prev, next);
      await refreshParts(DateTime.now(), set, { refreshAlerts: true });
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
    toggleAlert: async (alert) => {
      const now = DateTime.now();
      await createAlertEvent(
        alert.id,
        dateTimeToTicks(now),
        alert.status.tag === "ignored" ? "unignored" : "ignored",
      );
      await refreshParts(now, set, { refreshAlerts: true });
    },
  };
});

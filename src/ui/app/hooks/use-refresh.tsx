import type { Alert, App, Ref, Tag, Target } from "@/lib/entities";
import { refresh, useAppState } from "@/lib/state";
import { useMemo } from "react";

export function useRefresh() {
  const lastRefresh = useAppState((state) => state.lastRefresh);
  return {
    refreshToken: lastRefresh,
    refresh,
  };
}

export function useApps(appIds?: Ref<App>[]): App[] {
  const allApps = useAppState((state) => state.apps);
  const apps = useMemo(
    () =>
      (appIds
        ? appIds.map((id) => allApps[id])
        : Object.values(allApps)
      ).filter((app) => app !== undefined),
    [allApps, appIds],
  );
  return apps;
}

export function useApp(appId: Ref<App> | null): App | null {
  const allApps = useAppState((state) => state.apps);
  const app = useMemo(
    () => (appId ? (allApps[appId] ?? null) : null),
    [allApps, appId],
  );
  return app;
}

export function useTags(tagIds?: Ref<Tag>[]): Tag[] {
  const allTags = useAppState((state) => state.tags);
  const tags = useMemo(
    () =>
      (tagIds
        ? tagIds.map((id) => allTags[id])
        : Object.values(allTags)
      ).filter((tag) => tag !== undefined),
    [allTags, tagIds],
  );
  return tags;
}

export function useTag(tagId: Ref<Tag> | null): Tag | null {
  const allTags = useAppState((state) => state.tags);
  const tag = useMemo(
    () => (tagId ? (allTags[tagId] ?? null) : null),
    [allTags, tagId],
  );
  return tag;
}

export function useAlerts(alertIds?: Ref<Alert>[]): Alert[] {
  const allAlerts = useAppState((state) => state.alerts);
  const alerts = useMemo(
    () =>
      (alertIds
        ? alertIds.map((id) => allAlerts[id])
        : Object.values(allAlerts)
      ).filter((alert) => alert !== undefined),
    [allAlerts, alertIds],
  );
  return alerts;
}

export function useAlert(alertId: Ref<Alert> | null): Alert | null {
  const allAlerts = useAppState((state) => state.alerts);
  const alert = useMemo(
    () => (alertId ? (allAlerts[alertId] ?? null) : null),
    [allAlerts, alertId],
  );
  return alert;
}

export function useTargetApps(target?: Target | null): App[] | null {
  const app = useApp(target?.tag === "app" ? target.id : null);
  const tag = useTag(target?.tag === "tag" ? target.id : null);
  // Tag's list of Apps
  const tagApps = useApps(tag?.apps ?? []);
  if (!target) return null;
  return target.tag === "app" ? (app === null ? [] : [app]) : tagApps;
}

import type { Alert, App, Ref, Tag, Target } from "@/lib/entities";
import { refresh, useAppState } from "@/lib/state";
import { useCallback, useMemo } from "react";

export function useRefresh() {
  const lastRefresh = useAppState((state) => state.lastRefresh);
  // TODO: queue a debounced refresh?
  const handleStale = useCallback(function <T>(data: (T | undefined)[]) {
    return data.filter((x) => x !== undefined);
  }, []);

  return {
    refreshToken: lastRefresh,
    refresh,
    handleStaleApps: handleStale,
    handleStaleTags: handleStale,
    handleStaleAlerts: handleStale,
  };
}

export function useApps(appIds?: Ref<App>[]) {
  const allApps = useAppState((state) => state.apps);
  const { handleStaleApps } = useRefresh();
  const apps = useMemo(() => {
    const filteredApps = appIds
      ? appIds.map((id) => allApps[id])
      : Object.values(allApps);
    return handleStaleApps(filteredApps);
  }, [allApps, handleStaleApps, appIds]);
  return apps;
}

export function useApp(appId: Ref<App> | null): App | null {
  const allApps = useAppState((state) => state.apps);
  const { handleStaleApps } = useRefresh();
  const app = useMemo(() => {
    if (appId === null) return null;
    return handleStaleApps([allApps[appId]])[0];
  }, [allApps, handleStaleApps, appId]);
  return app;
}

export function useTags(tagIds?: Ref<Tag>[]) {
  const allTags = useAppState((state) => state.tags);
  const { handleStaleTags } = useRefresh();
  const tags = useMemo(() => {
    const filteredTags = tagIds
      ? tagIds.map((id) => allTags[id])
      : Object.values(allTags);
    return handleStaleTags(filteredTags);
  }, [allTags, handleStaleTags, tagIds]);
  return tags;
}

export function useTag(tagId: Ref<Tag> | null): Tag | null {
  const allTags = useAppState((state) => state.tags);
  const { handleStaleTags } = useRefresh();
  const tag = useMemo(() => {
    if (tagId === null) return null;
    return handleStaleTags([allTags[tagId]])[0];
  }, [allTags, handleStaleTags, tagId]);
  return tag;
}

export function useAlerts(alertIds?: Ref<Alert>[]) {
  const allAlerts = useAppState((state) => state.alerts);
  const { handleStaleAlerts } = useRefresh();
  const alerts = useMemo(() => {
    const filteredAlerts = alertIds
      ? alertIds.map((id) => allAlerts[id])
      : Object.values(allAlerts);
    return handleStaleAlerts(filteredAlerts);
  }, [allAlerts, handleStaleAlerts, alertIds]);
  return alerts;
}

export function useAlert(alertId: Ref<Alert> | null): Alert | null {
  const allAlerts = useAppState((state) => state.alerts);
  const { handleStaleAlerts } = useRefresh();
  const alert = useMemo(() => {
    if (alertId === null) return null;
    return handleStaleAlerts([allAlerts[alertId]])[0];
  }, [allAlerts, handleStaleAlerts, alertId]);
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

import type { Alert, App, Tag } from "@/lib/entities";
import { useAppState } from "@/lib/state";
import { matchSorter, type KeyOption } from "match-sorter";
import { useMemo, useState } from "react";

export function useSearch<T>(items: T[], paths: KeyOption<T>[]) {
  const [query, setQuery] = useState("");

  const filtered = useMemo(() => {
    if (query) {
      return matchSorter(items, query, {
        keys: paths,
      });
    } else {
      return items;
    }
    // paths is an array of objects, so we need to stringify it to compare.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [items, JSON.stringify(paths), query]);

  return [query, setQuery, filtered] as const;
}

export function useAppsSearch(apps: App[]) {
  return useSearch(apps, ["name", "company", "description"]);
}

export function useTagsSearch(tags: Tag[]) {
  return useSearch(tags, ["name"]);
}

export function useTargetsSearch(apps: App[], tags: Tag[]) {
  const [query, setAppQuery, filteredApps] = useAppsSearch(apps);
  const [, setTagQuery, filteredTags] = useTagsSearch(tags);
  function setQuery(query: string) {
    setAppQuery(query);
    setTagQuery(query);
  }
  return [query, setQuery, filteredApps, filteredTags] as const;
}

export function useAlertsSearch(alerts: Alert[]) {
  const allApps = useAppState((state) => state.apps);
  const allTags = useAppState((state) => state.tags);
  const infusedAlerts = useMemo(() => {
    return alerts.map((alert) => {
      // Apps and Tags will definitely be found, thus the !
      return {
        alert,
        app: alert.target.tag === "app" ? allApps[alert.target.id]! : undefined,
        tag: alert.target.tag === "tag" ? allTags[alert.target.id]! : undefined,
      };
    });
  }, [alerts, allApps, allTags]);
  const [query, setQuery, filteredAlerts] = useSearch(infusedAlerts, [
    "app.name",
    "app.company",
    "app.description",
    "tag.name",
  ]);
  const filteredAlertsInner = useMemo(
    () => filteredAlerts.map(({ alert }) => alert),
    [filteredAlerts],
  );
  return [query, setQuery, filteredAlertsInner] as const;
}

import { useHistoryState } from "@/hooks/use-history-state";
import type { Alert, App, Tag } from "@/lib/entities";
import { useAppState } from "@/lib/state";
import { matchSorter, type KeyOption } from "match-sorter";
import { useMemo } from "react";

export function useSearch<T>(
  items: T[],
  paths: KeyOption<T>[],
  key: string | undefined,
) {
  const [query, setQuery] = useHistoryState<string>("", key);

  const filtered = useMemo(() => {
    if (query) {
      return matchSorter(items, query, {
        keys: paths,
        /*
         * This theshold is higher than ACRONYM and MATCHES (default)
         * which are really bad for our purposes.
         */
        threshold: matchSorter.rankings.CONTAINS,
      });
    } else {
      return items;
    }
    // paths is an array of objects, so we need to stringify it to compare.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [items, JSON.stringify(paths), query]);

  return [query, setQuery, filtered] as const;
}

export function useAppsSearch(apps: App[], key: string | undefined) {
  return useSearch(
    apps,
    [
      "name",
      "company",
      "description",
      "identity.aumid",
      "identity.path",
      "identity.baseUrl",
      "identity.identifier",
      "identity.file",
    ],
    key,
  );
}

export function useTagsSearch(tags: Tag[], key: string | undefined) {
  return useSearch(tags, ["name"], key);
}

export function useTargetsSearch(
  apps: App[],
  tags: Tag[],
  key: string | undefined,
) {
  const [query, setAppQuery, filteredApps] = useAppsSearch(apps, key + "-app");
  const [, setTagQuery, filteredTags] = useTagsSearch(tags, key + "-tag");
  function setQuery(query: string) {
    setAppQuery(query);
    setTagQuery(query);
  }
  return [query, setQuery, filteredApps, filteredTags] as const;
}

export function useAlertsSearch(alerts: Alert[], key: string | undefined) {
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
  const [query, setQuery, filteredAlerts] = useSearch(
    infusedAlerts,
    [
      "app.name",
      "app.company",
      "app.description",
      "app.identity.aumid",
      "app.identity.path",
      "app.identity.baseUrl",
      "app.identity.identifier",
      "app.identity.file",

      "tag.name",
    ],
    key,
  );
  const filteredAlertsInner = useMemo(
    () => filteredAlerts.map(({ alert }) => alert),
    [filteredAlerts],
  );
  return [query, setQuery, filteredAlertsInner] as const;
}

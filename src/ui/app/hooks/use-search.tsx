import type { App, Tag } from "@/lib/entities";
import fuzzysort from "fuzzysort";
import _ from "lodash";
import { useMemo, useState } from "react";
import { type Path } from "react-hook-form";

export function useSearch<T>(items: T[], paths: Path<T>[]) {
  const [query, setQuery] = useState("");

  const filtered = useMemo(() => {
    if (query) {
      const results = fuzzysort.go(query, items, {
        keys: paths,
      });
      // ignore fuzzy-search sorting.
      return _.map(results, "obj");
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

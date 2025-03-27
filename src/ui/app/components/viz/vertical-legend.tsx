import { useApps, useTags } from "@/hooks/use-refresh";
import type { App, Ref, Tag } from "@/lib/entities";
import {
  useCallback,
  useMemo,
  useState,
  type Dispatch,
  type SetStateAction,
  type CSSProperties,
  useEffect,
  useRef,
} from "react";
import {
  TagIcon,
  ChevronDown,
  ChevronRight,
  EllipsisVertical,
} from "lucide-react";
import { Text } from "@/components/ui/text";
import { cn } from "@/lib/utils";
import AppIcon from "@/components/app/app-icon";
import _ from "lodash";
import { Checkbox } from "@/components/ui/checkbox";
import type { ClassValue } from "clsx";
import { SearchBar } from "@/components/search-bar";
import { useAppsSearch } from "@/hooks/use-search";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import type { CheckedState } from "@radix-ui/react-checkbox";
import { ScoreCircle } from "@/components/tag/score";
import { VariableSizeList as List } from "react-window";
import AutoSizer from "react-virtualized-auto-sizer";
import { untagged } from "@/lib/state";

type ListItem = {
  id: string;
  type: "tag" | "app" | "spacer";
  tag?: Tag;
  app?: App;
  isExpanded: boolean;
  nestingLevel: number;
};

export function VerticalLegend({
  appIds,
  className,
  uncheckedApps,
  setUncheckedApps,
}: {
  appIds?: Ref<App>[];
  uncheckedApps: Record<Ref<App>, boolean>;
  setUncheckedApps: Dispatch<SetStateAction<Record<Ref<App>, boolean>>>;
  className?: ClassValue;
}) {
  const apps = useApps(appIds);
  const [query, setQuery, filteredApps] = useAppsSearch(apps);
  const tagIds = useMemo(() => {
    return _(filteredApps)
      .map((app) => app.tagId)
      .filter((tagId) => tagId !== null)
      .uniq()
      .value();
  }, [filteredApps]);
  const tags = useTags(tagIds);

  // State for expanded/collapsed tags
  const [unexpandedTags, setUnexpandedTags] = useState<Record<string, boolean>>(
    {},
  );

  // Toggle expansion state for a tag
  const toggleExpandTag = useCallback(
    (tagId: string) => {
      setUnexpandedTags((prev) => ({
        ...prev,
        [tagId]: !prev[tagId],
      }));
    },
    [setUnexpandedTags],
  );

  const checkApp = useCallback(
    (id: Ref<App>, checked: boolean) => {
      setUncheckedApps((prev) => ({
        ...prev,
        [id]: !checked,
      }));
    },
    [setUncheckedApps],
  );

  const checkTag = useCallback(
    (id: Ref<Tag>, checked: boolean) => {
      setUncheckedApps((prev) => {
        const newState = { ...prev };
        apps
          .filter((app) => app.tagId === id)
          .forEach((app) => {
            newState[app.id] = !checked;
          });
        return newState;
      });
    },
    [setUncheckedApps, apps],
  );

  const showUntagged = useMemo(() => {
    return filteredApps.some((app) => app.tagId === null);
  }, [filteredApps]);

  const selectAllApps = () => {
    setUncheckedApps({});
  };

  const unselectAllApps = () => {
    setUncheckedApps(() => {
      const newState: Record<Ref<App>, boolean> = {};
      apps.forEach((app) => {
        newState[app.id] = true;
      });
      return newState;
    });
  };

  const selectAllFilteredApps = () => {
    setUncheckedApps((prev) => {
      const newState = { ...prev };
      filteredApps.forEach((app) => {
        newState[app.id] = false;
      });
      return newState;
    });
  };

  const unselectAllFilteredApps = () => {
    setUncheckedApps((prev) => {
      const newState = { ...prev };
      filteredApps.forEach((app) => {
        newState[app.id] = true;
      });
      return newState;
    });
  };

  const tagStatus = useCallback(
    (tagId: Ref<Tag>): CheckedState => {
      const allUnchecked = apps
        .filter((app) => app.tagId === (tagId === untagged.id ? null : tagId))
        .every((app) => !!uncheckedApps[app.id]);
      const allChecked = apps
        .filter((app) => app.tagId === (tagId === untagged.id ? null : tagId))
        .every((app) => !uncheckedApps[app.id]);
      return allChecked ? true : allUnchecked ? false : "indeterminate";
    },
    [apps, uncheckedApps],
  );

  // Prepare flattened list data
  const listData = useMemo(() => {
    const data: ListItem[] = [];

    // Add tags
    [...tags, ...(showUntagged ? [untagged] : [])].forEach((tag, index) => {
      // Add spacer between tag groups
      if (index > 0) {
        data.push({
          id: `spacer-${String(tag.id)}`,
          type: "spacer",
          isExpanded: false,
          nestingLevel: 0,
        });
      }

      const tagIdStr = tag.id + "";
      data.push({
        id: `tag-${tagIdStr}`,
        type: "tag",
        tag,
        isExpanded: !unexpandedTags[tagIdStr],
        nestingLevel: 0,
      });

      // Add apps under this tag if expanded
      if (!unexpandedTags[tagIdStr]) {
        filteredApps
          .filter((app) => app.tagId === tag.id)
          .forEach((app) => {
            data.push({
              id: `app-${app.id}`,
              type: "app",
              app,
              isExpanded: false,
              nestingLevel: 1,
            });
          });
      }
    });

    return data;
  }, [tags, showUntagged, unexpandedTags, filteredApps]);

  const varRef = useRef<List | null>(null);

  const ListItem = useCallback(
    ({ index, style }: { index: number; style: CSSProperties }) => {
      const item = listData[index];
      if (item.type === "spacer") {
        return <div style={style} className="h-4" />;
      }
      if (item.type === "tag") {
        const tag = item.tag!;
        const tagIdStr = tag.id + "";
        return (
          <div
            style={style}
            className="flex items-center gap-2 p-1 hover:bg-accent/50 rounded-md"
          >
            <button onClick={() => toggleExpandTag(tagIdStr)} className="p-1">
              {!unexpandedTags[tagIdStr] ? (
                <ChevronDown className="h-4 w-4" />
              ) : (
                <ChevronRight className="h-4 w-4" />
              )}
            </button>
            <Checkbox
              checked={tagStatus(tag.id)}
              onCheckedChange={(checked) => checkTag(tag.id, checked === true)}
              className="size-4 shrink-0 border-border data-[state=checked]:bg-border data-[state=checked]:text-foreground"
            />
            <div
              className="w-1 h-4 shrink-0 rounded-sm"
              style={{ backgroundColor: tag.color }}
            />
            <TagIcon
              className="h-4 w-4 shrink-0"
              style={{ color: tag.color }}
            />
            <Text className="text-sm">{tag.name}</Text>
            <ScoreCircle score={tag.score} />
          </div>
        );
      } else {
        const app = item.app!;
        return (
          <div
            style={style}
            className="flex items-center gap-2 pl-9 p-1 hover:bg-accent/50 rounded-md"
          >
            <Checkbox
              checked={!uncheckedApps[app.id]}
              onCheckedChange={(checked) => checkApp(app.id, checked === true)}
              className="size-4 shrink-0 border-border data-[state=checked]:bg-border data-[state=checked]:text-foreground"
            />
            <div
              className="w-1 h-4 shrink-0 rounded-sm"
              style={{ backgroundColor: app.color }}
            />
            <AppIcon buffer={app.icon} className="h-4 w-4 shrink-0" />
            <Text className="text-sm text-muted-foreground">{app.name}</Text>
          </div>
        );
      }
    },
    [
      listData,
      unexpandedTags,
      tagStatus,
      checkTag,
      checkApp,
      uncheckedApps,
      toggleExpandTag,
    ],
  );

  // Recalculate item sizes when list data changes
  useEffect(() => {
    varRef.current?.resetAfterIndex(0);
  }, [listData]);

  const itemSize = useCallback(
    (index: number) => {
      console.log(listData, index, listData[index]);
      const item = listData[index];
      if (item.type === "spacer") {
        return 8;
      }
      return item.type === "app" ? 28 : 32;
    },
    [listData],
  );

  return (
    <div className={cn("overflow-auto h-full", className)}>
      <div className="flex flex-col gap-2 h-full">
        <div className="flex items-center gap-2">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon">
                <EllipsisVertical />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              <DropdownMenuItem onClick={selectAllApps}>
                Select All
              </DropdownMenuItem>
              <DropdownMenuItem onClick={unselectAllApps}>
                Unselect All
              </DropdownMenuItem>
              {query && filteredApps.length > 0 && (
                <>
                  <DropdownMenuItem onClick={selectAllFilteredApps}>
                    Select All Filtered
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={unselectAllFilteredApps}>
                    Unselect All Filtered
                  </DropdownMenuItem>
                </>
              )}
            </DropdownMenuContent>
          </DropdownMenu>
          <SearchBar
            placeholder="Filter"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
        <div className="flex-1">
          <AutoSizer>
            {({ height, width }) => (
              <List
                ref={varRef}
                height={height}
                width={width}
                itemSize={itemSize}
                itemCount={listData.length}
                itemData={listData}
              >
                {ListItem}
              </List>
            )}
          </AutoSizer>
        </div>
      </div>
    </div>
  );
}

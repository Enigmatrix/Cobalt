import AppIcon from "@/components/app/app-icon";
import { AppBadge } from "@/components/app/app-list-item";
import { NoApps, NoAppsFound } from "@/components/empty-states";
import { SearchBar } from "@/components/search-bar";
import { ScoreCircle } from "@/components/tag/score";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Text } from "@/components/ui/text";
import { useApps, useTag } from "@/hooks/use-refresh";
import { useAppsSearch } from "@/hooks/use-search";
import type { App, Ref, Tag } from "@/lib/entities";
import { cn } from "@/lib/utils";
import { PopoverAnchor } from "@radix-ui/react-popover";
import { PlusIcon } from "lucide-react";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import AutoSizer from "react-virtualized-auto-sizer";
import { VariableSizeList as List } from "react-window";
import { Checkbox } from "../ui/checkbox";

export function MiniTagItem({ tagId }: { tagId: Ref<Tag> | null }) {
  const tag = useTag(tagId);
  return (
    tag && (
      <Badge
        variant="outline"
        style={{
          borderColor: tag.color,
          color: tag.color,
          backgroundColor: "rgba(255, 255, 255, 0.2)",
        }}
        className="whitespace-nowrap min-w-0 -my-0.5 px-2 py-0.5 rounded-full border"
      >
        <Text className="max-w-32">{tag.name}</Text>
        <ScoreCircle score={tag.score} className="ml-2 -mr-1" />
      </Badge>
    )
  );
}

export function ChooseMultiApps({
  placeholder,
  value,
  onValueChanged,
}: {
  placeholder?: ReactNode;
  value: Ref<App>[];
  onValueChanged: (value: Ref<App>[]) => void;
}) {
  const [open, setOpenInner] = useState(false);
  const allApps = useApps();
  const valueApps = useApps(value);

  const [query, setQuery, filteredApps] = useAppsSearch(allApps, undefined);
  const setOpen = useCallback(
    (open: boolean) => {
      setOpenInner(open);
      if (open) setQuery("");
    },
    [setOpenInner, setQuery],
  );

  const toggleOption = useCallback(
    (option: Ref<App>) => {
      const newSelectedValues = value.includes(option)
        ? value.filter((value) => value !== option)
        : [...value, option];
      onValueChanged(newSelectedValues);
    },
    [value, onValueChanged],
  );

  const items = useMemo(() => {
    if (allApps.length === 0) {
      return [
        { height: 104, item: <NoApps variant="small" className="m-auto" /> },
      ];
    }
    if (allApps.length !== 0 && filteredApps.length === 0) {
      return [
        {
          height: 104,
          item: <NoAppsFound variant="small" className="m-auto" />,
        },
      ];
    }

    return filteredApps.map((app) => {
      const isSelected = value.includes(app.id);
      return {
        height: 32,
        item: (
          <div
            key={app.id}
            className={cn("flex items-center gap-2 h-8 px-2 text-sm", {
              "bg-muted/80": isSelected,
            })}
          >
            <Checkbox
              checked={isSelected}
              onCheckedChange={() => toggleOption(app.id)}
              className="border-foreground/20"
            />
            <AppIcon
              appIcon={app.icon}
              className="mr-2 h-4 w-4 text-muted-foreground"
            />
            <Text>{app.name}</Text>
            {/* Don't show tag if for *this* tag, since a creating tag will not even be valid */}
            {!isSelected && <MiniTagItem tagId={app.tagId} />}
          </div>
        ),
      };
    });
  }, [filteredApps, allApps, value, toggleOption]);

  const listRef = useRef<List>(null);

  // Recalculate item sizes when list data changes
  useEffect(() => {
    listRef.current?.resetAfterIndex(0);
  }, [items]);

  return (
    <Popover open={open} onOpenChange={setOpen} modal>
      <div className="flex flex-wrap items-center gap-2">
        {valueApps.map((app, index) => {
          return (
            <div className="flex items-center flex-nowrap min-w-0" key={app.id}>
              <AppBadge app={app} remove={() => toggleOption(app.id)} />
              {valueApps.length === index + 1 && (
                <>
                  <div className="ml-2 border-l h-6" />
                  <PopoverAnchor>
                    <PopoverTrigger asChild>
                      <Button variant="ghost" size="icon" className="w-8 h-8">
                        <PlusIcon className="text-muted-foreground" />
                      </Button>
                    </PopoverTrigger>
                  </PopoverAnchor>
                </>
              )}
            </div>
          );
        })}
        {valueApps.length === 0 && (
          <div className="flex flex-wrap items-center text-muted-foreground h-8">
            {placeholder ?? <Text>No apps in tag. Add some!</Text>}
            <div className="ml-2 border-l h-6" />
            <PopoverTrigger asChild>
              <Button variant="ghost" size="icon" className="w-8 h-8">
                <PlusIcon />
              </Button>
            </PopoverTrigger>
          </div>
        )}
      </div>
      <PopoverContent className="p-0 bg-card h-80 flex flex-col">
        <SearchBar
          placeholder="Search..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />

        <div style={{ flex: "1 1 auto" }}>
          <AutoSizer>
            {({ height, width }) => (
              <List
                ref={listRef}
                itemSize={(index) => items[index].height}
                itemCount={items.length}
                itemData={items}
                height={height}
                width={width}
              >
                {({ index, style }) => {
                  const item = items[index];
                  return <div style={style}>{item.item}</div>;
                }}
              </List>
            )}
          </AutoSizer>
        </div>
      </PopoverContent>
    </Popover>
  );
}

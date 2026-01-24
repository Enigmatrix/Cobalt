import AppIcon from "@/components/app/app-icon";
import { MiniTagItem } from "@/components/app/choose-multi-apps";
import {
  NoApps,
  NoAppsFound,
  NoTags,
  NoTagsFound,
} from "@/components/empty-states";
import { SearchBar } from "@/components/search-bar";
import { CreateTagDialog } from "@/components/tag/create-tag-dialog";
import { ScoreCircle } from "@/components/tag/score";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Text } from "@/components/ui/text";
import { useApp, useApps, useTag, useTags } from "@/hooks/use-refresh";
import { useTargetsSearch } from "@/hooks/use-search";
import type { Target } from "@/lib/entities";
import type { tagSchema } from "@/lib/schema";
import { useAppState } from "@/lib/state";
import { cn } from "@/lib/utils";
import { useConcatVirtualItems, useVirtualSection } from "@/lib/virtualization";
import { ChevronDown, ChevronRight, Plus, TagIcon } from "lucide-react";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ComponentProps,
  type Dispatch,
  type ReactNode,
  type SetStateAction,
} from "react";
import AutoSizer from "react-virtualized-auto-sizer";
import { VariableSizeList as List } from "react-window";
import type { z } from "zod";

export function ChooseTarget({
  onValueChanged: onValueChangedInner,
  ...props
}: ComponentProps<typeof ChooseTargetTrigger> & {
  onValueChanged: (value: Target) => void;
}) {
  const [open, setOpenInner] = useState(false);
  const createTag = useAppState((state) => state.createTag);
  const allTags = useTags();
  const allApps = useApps();

  const [query, setQuery, filteredApps, filteredTags] = useTargetsSearch(
    allApps,
    allTags,
    undefined,
  );
  const setOpen = useCallback(
    (open: boolean) => {
      setOpenInner(open);
      if (open) setQuery("");
    },
    [setOpenInner, setQuery],
  );

  const onValueChanged = useCallback(
    (val: Target) => {
      onValueChangedInner(val);
      setOpen(false);
    },
    [onValueChangedInner, setOpen],
  );

  const onTagCreate = useCallback(
    async (tag: z.infer<typeof tagSchema>) => {
      const tagId = await createTag(tag);
      onValueChanged({ tag: "tag", id: tagId });
    },
    [createTag, onValueChanged],
  );

  const tagInnerItems = useMemo(() => {
    if (allTags.length === 0) {
      return [
        { height: 104, item: <NoTags variant="small" className="m-auto" /> },
      ];
    }
    if (allTags.length !== 0 && filteredTags.length === 0) {
      return [
        {
          height: 104,
          item: <NoTagsFound variant="small" className="m-auto" />,
        },
      ];
    }
    return filteredTags.map((tag) => ({
      height: 32,
      item: (
        <button
          key={tag.id}
          className={cn(
            "flex flex-1 gap-2 items-center h-8 hover:bg-muted/90 px-2 mx-2 rounded-md text-sm",
            {
              "bg-muted/60":
                props.value?.tag === "tag" && props.value?.id === tag.id,
            },
          )}
          onClick={() => onValueChanged({ tag: "tag", id: tag.id })}
        >
          <TagIcon className="w-4 h-4 shrink-0" style={{ color: tag.color }} />
          <Text>{tag.name}</Text>
          <ScoreCircle score={tag.score} />
        </button>
      ),
    }));
  }, [filteredTags, allTags, onValueChanged, props.value]);

  const tagItems = useMemo(() => {
    return [
      ...tagInnerItems,
      {
        height: 32,
        item: (
          <div className="w-full px-2">
            <CreateTagDialog
              trigger={
                <Button variant="outline" size="sm" className="w-full p-2">
                  <Plus className="w-4 h-4" />
                  Create Tag
                </Button>
              }
              onSubmit={onTagCreate}
            />
          </div>
        ),
      },
    ];
  }, [tagInnerItems, onTagCreate]);

  const createTagHeading = useMemo(
    () => (open: boolean, setOpen: Dispatch<SetStateAction<boolean>>) => {
      return {
        height: 40,
        item: <SectionHeader open={open} setOpen={setOpen} heading="Tags" />,
      };
    },
    [],
  );

  const { output: tagSection } = useVirtualSection({
    heading: createTagHeading,
    items: tagItems,
    initialOpen: true,
  });

  const createAppHeading = useMemo(
    () => (open: boolean, setOpen: Dispatch<SetStateAction<boolean>>) => {
      return {
        height: 40,
        item: <SectionHeader open={open} setOpen={setOpen} heading="Apps" />,
      };
    },
    [],
  );

  const appInnerItems = useMemo(() => {
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

    return filteredApps.map((app) => ({
      height: 32,
      item: (
        <button
          key={app.id}
          className={cn(
            "flex flex-1 gap-2 items-center h-8 hover:bg-muted/90 px-2 mx-2 rounded-md text-sm",
            {
              "bg-muted/60":
                props.value?.tag === "app" && props.value?.id === app.id,
            },
          )}
          onClick={() => onValueChanged({ tag: "app", id: app.id })}
        >
          <AppIcon appIcon={app.icon} className="w-4 h-4 shrink-0" />
          <Text>{app.name}</Text>
          <MiniTagItem tagId={app.tagId} />
        </button>
      ),
    }));
  }, [filteredApps, allApps, onValueChanged, props.value]);

  const { output: appSection } = useVirtualSection({
    heading: createAppHeading,
    items: appInnerItems,
    initialOpen: true,
  });

  const items = useConcatVirtualItems(tagSection, appSection);

  const listRef = useRef<List>(null);

  // Recalculate item sizes when list data changes
  useEffect(() => {
    listRef.current?.resetAfterIndex(0);
  }, [items]);

  return (
    <Popover open={open} onOpenChange={setOpen} modal>
      <PopoverTrigger asChild>
        <ChooseTargetTrigger {...props} />
      </PopoverTrigger>
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
                  return (
                    <div className="flex" style={style}>
                      {item.item}
                    </div>
                  );
                }}
              </List>
            )}
          </AutoSizer>
        </div>
      </PopoverContent>
    </Popover>
  );
}

function ChooseTargetTrigger({
  value,
  className,
  placeholder,
  ...props
}: Omit<ComponentProps<typeof Button>, "value"> & {
  value: Target | undefined;
  placeholder?: ReactNode;
}) {
  const app = useApp(value?.tag === "app" ? value.id : null);
  const tag = useTag(value?.tag === "tag" ? value.id : null);
  return (
    <Button
      variant="outline"
      {...props}
      className={cn("flex gap-2 items-center", className)}
    >
      {value?.tag === "app" && app ? (
        <>
          <AppIcon appIcon={app.icon} className="w-5 h-5 shrink-0" />
          <Text>{app.name}</Text>
        </>
      ) : value?.tag === "tag" && tag ? (
        <>
          <TagIcon className="w-5 h-5 shrink-0" style={{ color: tag.color }} />
          <Text>{tag.name}</Text>
          <ScoreCircle score={tag.score} />
        </>
      ) : (
        (placeholder ?? (
          <Text className="text-muted-foreground">Choose Target</Text>
        ))
      )}
    </Button>
  );
}

function SectionHeader({
  open,
  setOpen,
  heading,
}: {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
  heading: string;
}) {
  return (
    <div className="w-full px-2 pt-1.5">
      <button
        className="flex w-full gap-2 h-8 text-sm text-muted-foreground p-2 bg-secondary items-center rounded-t-sm"
        onClick={() => setOpen((o) => !o)}
      >
        {open ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
        <Text>{heading}</Text>
      </button>
    </div>
  );
}

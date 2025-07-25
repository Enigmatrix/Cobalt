import AppIcon from "@/components/app/app-icon";
import { MiniTagItem } from "@/components/app/choose-multi-apps";
import {
  NoApps,
  NoAppsFound,
  NoTags,
  NoTagsFound,
} from "@/components/empty-states";
import { CreateTagDialog } from "@/components/tag/create-tag-dialog";
import { ScoreCircle } from "@/components/tag/score";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
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
import { ChevronDown, ChevronRight, Plus, TagIcon } from "lucide-react";
import {
  useCallback,
  useState,
  type ComponentProps,
  type ReactNode,
} from "react";
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

  const [, setQuery, filteredApps, filteredTags] = useTargetsSearch(
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

  return (
    <Popover open={open} onOpenChange={setOpen} modal>
      <PopoverTrigger asChild>
        <ChooseTargetTrigger {...props} />
      </PopoverTrigger>
      <PopoverContent className="p-0">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Search..."
            onValueChange={(val) => setQuery(val.toLowerCase())}
          />
          <CommandList>
            <CommandItem value="-" className="hidden" />
            <ToggleableCommandGroup heading="Tags">
              {filteredTags.map((tag) => (
                <CommandItem
                  key={tag.id}
                  value={`tag-${tag.id}`}
                  className={cn({
                    "bg-muted/60":
                      props.value?.tag === "tag" && props.value?.id === tag.id,
                  })}
                  onSelect={() => onValueChanged({ tag: "tag", id: tag.id })}
                >
                  <TagIcon
                    className="w-4 h-4 shrink-0"
                    style={{ color: tag.color }}
                  />
                  <Text>{tag.name}</Text>
                  <ScoreCircle score={tag.score} />
                </CommandItem>
              ))}
              {allTags.length === 0 && (
                <NoTags variant="small" className="m-auto" />
              )}
              {allTags.length !== 0 && filteredTags.length === 0 && (
                <NoTagsFound variant="small" className="m-auto" />
              )}
              <CreateTagDialog
                trigger={
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full p-0 mt-2"
                  >
                    <Plus className="w-4 h-4" />
                    Create Tag
                  </Button>
                }
                onSubmit={onTagCreate}
              />
            </ToggleableCommandGroup>

            <ToggleableCommandGroup heading="Apps">
              {filteredApps.map((app) => (
                <CommandItem
                  key={app.id}
                  value={`app-${app.id}`}
                  className={cn({
                    "bg-muted/60":
                      props.value?.tag === "app" && props.value?.id === app.id,
                  })}
                  onSelect={() => onValueChanged({ tag: "app", id: app.id })}
                >
                  <AppIcon appIcon={app.icon} className="w-4 h-4 shrink-0" />
                  <Text>{app.name}</Text>
                  <MiniTagItem tagId={app.tagId} />
                </CommandItem>
              ))}
              {allApps.length === 0 && (
                <NoApps variant="small" className="m-auto" />
              )}
              {allApps.length !== 0 && filteredApps.length === 0 && (
                <NoAppsFound variant="small" className="m-auto" />
              )}
            </ToggleableCommandGroup>
          </CommandList>
        </Command>
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
        (placeholder ?? <Text>Choose Target</Text>)
      )}
    </Button>
  );
}

function ToggleableCommandGroup({
  children,
  heading,
  ...props
}: ComponentProps<typeof CommandGroup> & { heading: string }) {
  const [open, setOpen] = useState(true);
  return (
    <CommandGroup
      {...props}
      heading={
        <div className="bg-secondary -my-1 -mx-2 flex rounded-t-sm">
          <button
            className="flex flex-1 gap-2 p-2"
            onClick={() => setOpen((open) => !open)}
          >
            {open ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
            <Text>{heading}</Text>
          </button>
        </div>
      }
    >
      {open && children}
    </CommandGroup>
  );
}

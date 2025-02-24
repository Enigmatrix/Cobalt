import {
  useCallback,
  useState,
  type ComponentProps,
  type ReactNode,
} from "react";
import {
  Popover,
  PopoverTrigger,
  PopoverContent,
} from "@/components/ui/popover";
import {
  Command,
  CommandInput,
  CommandList,
  CommandItem,
  CommandGroup,
} from "@/components/ui/command";
import { useApp, useApps, useTag, useTags } from "@/hooks/use-refresh";
import { Text } from "@/components/ui/text";
import { ChevronDown, ChevronRight, Plus, TagIcon } from "lucide-react";
import AppIcon from "@/components/app/app-icon";
import { useSearch } from "@/hooks/use-search";
import { CreateTagDialog } from "@/components/tag/create-tag-dialog";
import { Button, type ButtonProps } from "@/components/ui/button";
import type { Target } from "@/lib/entities";
import { useAppState } from "@/lib/state";
import { cn } from "@/lib/utils";

export function ChooseTarget({
  onValueChanged: onValueChangedInner,
  ...props
}: ComponentProps<typeof ChooseTargetTrigger> & {
  onValueChanged: (value: Target) => void;
}) {
  const [open, setOpen] = useState(false);
  const createTag = useAppState((state) => state.createTag);
  const allTags = useTags();
  const allApps = useApps();

  // TODO: should reset search value after close
  const [, setAppSearch, filteredApps] = useSearch(allApps, [
    "name",
    "company",
    "description",
  ]);
  const [, setTagSearch, filteredTags] = useSearch(allTags, ["name"]);

  function setQuery(val: string) {
    setAppSearch(val);
    setTagSearch(val);
  }

  const onValueChanged = useCallback(
    (val: Target) => {
      onValueChangedInner(val);
      setOpen(false);
    },
    [onValueChangedInner, setOpen],
  );

  const onTagCreate = useCallback(
    async (tag: { name: string; color: string }) => {
      const tagId = await createTag(tag);
      onValueChanged({ tag: "Tag", id: tagId });
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
              {/* TODO No tags empty state */}
              {filteredTags.map((tag) => (
                <CommandItem
                  key={tag.id}
                  value={tag.id.toString()}
                  onSelect={() => onValueChanged({ tag: "Tag", id: tag.id })}
                >
                  <TagIcon
                    className="w-4 h-4 shrink-0"
                    style={{ color: tag.color }}
                  />
                  <Text>{tag.name}</Text>
                </CommandItem>
              ))}
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
                  value={app.id.toString()}
                  onSelect={() => onValueChanged({ tag: "App", id: app.id })}
                >
                  <AppIcon buffer={app.icon} className="w-4 h-4 shrink-0" />
                  <Text>{app.name}</Text>
                </CommandItem>
              ))}
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
}: Omit<ButtonProps, "value"> & {
  value: Target | undefined;
  placeholder?: ReactNode;
}) {
  const app = useApp(value?.tag === "App" ? value.id : null);
  const tag = useTag(value?.tag === "Tag" ? value.id : null);
  return (
    <Button
      variant="outline"
      {...props}
      className={cn("flex gap-2 items-center", className)}
    >
      {value?.tag === "App" && app ? (
        <>
          <AppIcon buffer={app.icon} className="w-5 h-5 shrink-0" />
          <Text>{app.name}</Text>
        </>
      ) : value?.tag === "Tag" && tag ? (
        <>
          <TagIcon className="w-5 h-5 shrink-0" style={{ color: tag.color }} />
          <Text>{tag.name}</Text>
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

import { useCallback, useState, type ReactNode } from "react";
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
} from "@/components/ui/command";
import { useApps, useTag } from "@/hooks/use-refresh";
import { Text } from "@/components/ui/text";
import { CheckIcon, PlusIcon } from "lucide-react";
import { useAppsSearch } from "@/hooks/use-search";
import type { App, Ref, Tag } from "@/lib/entities";
import { cn } from "@/lib/utils";
import { NoApps, NoAppsFound } from "@/components/empty-states";
import AppIcon from "@/components/app/app-icon";
import { AppBadge } from "@/components/app/app-list-item";
import { PopoverAnchor } from "@radix-ui/react-popover";
import { Button } from "@/components/ui/button";
import { ScoreCircle } from "@/components/tag/score";
import { Badge } from "@/components/ui/badge";

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

  const [, setQuery, filteredApps] = useAppsSearch(allApps);
  const setOpen = useCallback(
    (open: boolean) => {
      setOpenInner(open);
      if (open) setQuery("");
    },
    [setOpenInner, setQuery],
  );

  const toggleOption = (option: Ref<App>) => {
    const newSelectedValues = value.includes(option)
      ? value.filter((value) => value !== option)
      : [...value, option];
    onValueChanged(newSelectedValues);
  };

  return (
    <Popover open={open} onOpenChange={setOpen} modal>
      <PopoverAnchor>
        <div className="flex flex-wrap items-center gap-2">
          {valueApps.map((app, index) => {
            return (
              <div
                className="flex items-center flex-nowrap min-w-0"
                key={app.id}
              >
                <AppBadge app={app} remove={() => toggleOption(app.id)} />
                {valueApps.length === index + 1 && (
                  <>
                    <div className="ml-2 border-l h-6" />
                    <PopoverTrigger asChild>
                      <Button variant="ghost" size="icon" className="w-8 h-8">
                        <PlusIcon className="text-muted-foreground" />
                      </Button>
                    </PopoverTrigger>
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
      </PopoverAnchor>
      <PopoverContent className="p-0">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Search..."
            onValueChange={(val) => setQuery(val.toLowerCase())}
          />
          <CommandList>
            <CommandItem value="-" className="hidden" />
            {filteredApps.map((app) => {
              const isSelected = value.indexOf(app.id) !== -1;
              return (
                <CommandItem
                  key={app.id}
                  value={`app-${app.id}`}
                  onSelect={() => toggleOption(app.id)}
                  className={cn({ "bg-muted/60": isSelected })}
                >
                  <div
                    className={cn(
                      "mr-2 flex h-4 w-4 items-center justify-center rounded-sm border border-primary",
                      isSelected
                        ? "bg-primary text-primary-foreground"
                        : "opacity-50 [&_svg]:invisible",
                    )}
                  >
                    <CheckIcon className="h-4 w-4" />
                  </div>
                  <AppIcon
                    buffer={app.icon}
                    className="mr-2 h-4 w-4 text-muted-foreground"
                  />
                  <Text>{app.name}</Text>
                  {/* Don't show tag if for *this* tag, since a creating tag will not even be valid */}
                  {!isSelected && <MiniTagItem tagId={app.tagId} />}
                </CommandItem>
              );
            })}

            {allApps.length === 0 && (
              <NoApps variant="small" className="m-auto" />
            )}
            {allApps.length !== 0 && filteredApps.length === 0 && (
              <NoAppsFound variant="small" className="m-auto" />
            )}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

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
import { useTags } from "@/hooks/use-refresh";
import { Text } from "@/components/ui/text";
import { Plus, TagIcon } from "lucide-react";
import { useTagsSearch } from "@/hooks/use-search";
import { CreateTagDialog } from "@/components/tag/create-tag-dialog";
import { Button } from "@/components/ui/button";
import type { Ref, Tag } from "@/lib/entities";
import { useAppState } from "@/lib/state";
import { cn } from "@/lib/utils";
import { NoTags, NoTagsFound } from "@/components/empty-states";
import type { z } from "zod";
import type { tagSchema } from "@/lib/schema";

export function ChooseTag({
  value,
  render,
  onValueChanged: onValueChangedInner,
}: {
  value: Ref<Tag> | null;
  onValueChanged: (value: Ref<Tag> | null) => void;
  render: (tagId: Ref<Tag> | null) => ReactNode;
}) {
  const [open, setOpenInner] = useState(false);
  const createTag = useAppState((state) => state.createTag);
  const allTags = useTags();

  const [, setQuery, filteredTags] = useTagsSearch(allTags);
  const setOpen = useCallback(
    (open: boolean) => {
      setOpenInner(open);
      if (open) setQuery("");
    },
    [setOpenInner, setQuery],
  );

  const onValueChanged = useCallback(
    (val: Ref<Tag> | null) => {
      onValueChangedInner(val);
      setOpen(false);
    },
    [onValueChangedInner, setOpen],
  );

  const onTagCreate = useCallback(
    async (tag: z.infer<typeof tagSchema>) => {
      const tagId = await createTag(tag);
      onValueChanged(tagId);
    },
    [createTag, onValueChanged],
  );

  return (
    <Popover open={open} onOpenChange={setOpen} modal>
      <PopoverTrigger asChild>{render(value)}</PopoverTrigger>
      <PopoverContent className="p-0">
        <Command shouldFilter={false}>
          <CommandInput
            placeholder="Search..."
            onValueChange={(val) => setQuery(val.toLowerCase())}
          />
          <CommandList>
            <CommandItem value="-" className="hidden" />
            {filteredTags.map((tag) => (
              <CommandItem
                key={tag.id}
                value={`tag-${tag.id}`}
                onSelect={() => onValueChanged(tag.id)}
                className={cn({ "bg-muted/60": value === tag.id })}
              >
                <TagIcon
                  className="w-4 h-4 shrink-0"
                  style={{ color: tag.color }}
                />
                <Text>{tag.name}</Text>
              </CommandItem>
            ))}

            {allTags.length === 0 && (
              <NoTags variant="small" className="m-auto" />
            )}
            {allTags.length !== 0 && filteredTags.length === 0 && (
              <NoTagsFound variant="small" className="m-auto" />
            )}
            <div className="p-1 flex items-center justify-end gap-1">
              <Button
                variant="outline"
                size="sm"
                className={cn({
                  hidden: value === null,
                })}
                onClick={() => onValueChanged(null)}
              >
                Clear
              </Button>

              <CreateTagDialog
                trigger={
                  <Button variant="outline" size="sm">
                    <Plus className="w-4 h-4" />
                    Create Tag
                  </Button>
                }
                onSubmit={onTagCreate}
              />
            </div>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}

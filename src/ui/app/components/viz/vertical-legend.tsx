import { useApps, useTags } from "@/hooks/use-refresh";
import type { App, Ref, Tag } from "@/lib/entities";
import { useMemo, useState, type Dispatch, type SetStateAction } from "react";
import { TagIcon, ChevronDown, ChevronRight } from "lucide-react";
import { Text } from "@/components/ui/text";
import { cn } from "@/lib/utils";
import AppIcon from "@/components/app/app-icon";
import _ from "lodash";
import { Checkbox } from "@/components/ui/checkbox";
import type { ClassValue } from "clsx";

const Untagged = Symbol("untagged");
export type AppTagId = Ref<Tag> | typeof Untagged;

export function VerticalLegend({
  appIds,
  className,
  uncheckedApps,
  setUncheckedApps,
  uncheckedTags,
  setUncheckedTags,
}: {
  appIds?: Ref<App>[];
  uncheckedApps: Record<Ref<App>, boolean>;
  setUncheckedApps: Dispatch<SetStateAction<Record<Ref<App>, boolean>>>;
  uncheckedTags: Record<AppTagId, boolean>;
  setUncheckedTags: Dispatch<SetStateAction<Record<AppTagId, boolean>>>;
  className?: ClassValue;
}) {
  const apps = useApps(appIds);
  const tagIds = useMemo(() => {
    return _(apps)
      .map((app) => app.tag_id)
      .filter((tagId) => tagId !== null)
      .uniq()
      .value();
  }, [apps]);
  const tags = useTags(tagIds);

  // State for expanded/collapsed tags
  const [unexpandedTags, setUnexpandedTags] = useState<Record<string, boolean>>(
    {},
  );

  // Toggle expansion state for a tag
  const toggleExpandTag = (tagId: string) => {
    setUnexpandedTags((prev) => ({
      ...prev,
      [tagId]: !prev[tagId],
    }));
  };

  const checkApp = (id: Ref<App>, checked: boolean) => {
    setUncheckedApps((prev) => ({
      ...prev,
      [id]: !checked,
    }));
  };

  const checkTag = (id: AppTagId, checked: boolean) => {
    setUncheckedTags((prev) => ({
      ...prev,
      [id]: !checked,
    }));
    setUncheckedApps((prev) => {
      const newState = { ...prev };
      apps
        .filter((app) => app.tag_id === (id === Untagged ? null : +id))
        .forEach((app) => {
          newState[app.id] = !checked;
        });
      return newState;
    });
  };

  return (
    <div className={cn("overflow-auto h-full", className)}>
      <div className="flex flex-col gap-2">
        {/* Tags with their apps */}
        {[
          ...tags,
          { id: Untagged, name: "Untagged", color: "#000000" } as const,
        ].map((tag) => {
          const tagIdStr = tag.id.toString();
          return (
            <div key={tagIdStr} className="flex flex-col">
              {/* Tag item */}
              <div className="flex items-center gap-2 p-1 hover:bg-accent/50 rounded-md cursor-pointer">
                <button
                  onClick={() => toggleExpandTag(tagIdStr)}
                  className="p-1"
                >
                  {!unexpandedTags[tagIdStr] ? (
                    <ChevronDown className="h-4 w-4" />
                  ) : (
                    <ChevronRight className="h-4 w-4" />
                  )}
                </button>
                <Checkbox
                  checked={!uncheckedTags[tag.id]}
                  onCheckedChange={(checked) =>
                    checkTag(tag.id, checked === true)
                  }
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
              </div>

              {/* Apps under this tag */}
              {!unexpandedTags[tagIdStr] &&
                apps
                  .filter(
                    (app) =>
                      app.tag_id === (tag.id === Untagged ? null : tag.id),
                  )
                  .map((app) => {
                    const appIdStr = app.id.toString();
                    return (
                      <div
                        key={appIdStr}
                        className="flex items-center gap-2 pl-9 p-1 hover:bg-accent/50 rounded-md cursor-pointer"
                      >
                        <Checkbox
                          checked={!uncheckedApps[app.id]}
                          onCheckedChange={(checked) =>
                            checkApp(app.id, checked === true)
                          }
                          className="size-4 shrink-0 border-border data-[state=checked]:bg-border data-[state=checked]:text-foreground"
                        />
                        <div
                          className="w-1 h-4 shrink-0 rounded-sm"
                          style={{ backgroundColor: app.color }}
                        />
                        <AppIcon
                          buffer={app.icon}
                          className="h-4 w-4 shrink-0"
                        />
                        <Text className="text-sm">{app.name}</Text>
                      </div>
                    );
                  })}
            </div>
          );
        })}
      </div>
    </div>
  );
}

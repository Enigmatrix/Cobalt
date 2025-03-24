import React, { useMemo } from "react";
import { cn } from "@/lib/utils";
import _ from "lodash";
import type { Ref, Tag } from "@/lib/entities";
import { type EntityMap } from "@/lib/state";
import { DateTime } from "luxon";
import { DurationText } from "@/components/time/duration-text";
import { DateTimeText } from "@/components/time/time-text";
import { useTags } from "@/hooks/use-refresh";
import { TagIcon } from "lucide-react";
import { Text } from "@/components/ui/text";

function HoverCard({
  tag,
  usageTicks,
  totalUsageTicks,
  at,
}: {
  tag: Tag;
  usageTicks: number;
  totalUsageTicks?: number;
  at: DateTime;
}) {
  return (
    <div className="flex items-center gap-2 py-2">
      <TagIcon
        className="w-6 h-6 shrink-0 ml-2 mr-1"
        style={{ color: tag.color }}
      />
      <div className="flex flex-col">
        <Text className="text-base max-w-52">{tag.name}</Text>
        <DateTimeText className="text-xs text-muted-foreground" datetime={at} />
      </div>

      <div className="flex-1"></div>
      <div className="flex flex-col items-end text-muted-foreground shrink-0 min-w-max">
        <DurationText className="font-semibold text-sm" ticks={usageTicks} />
        {totalUsageTicks && (
          <span className="inline-flex items-center gap-1 text-xs">
            <p>/</p>
            <DurationText ticks={totalUsageTicks} />
          </span>
        )}
      </div>
    </div>
  );
}

export const TagUsageChartTooltipContent = React.forwardRef<
  HTMLDivElement,
  React.ComponentProps<"div"> & {
    payload: EntityMap<Tag, number>;
    dt: DateTime;
    hideIndicator?: boolean;
    maximumTags?: number;
    hoveredTagId: Ref<Tag> | null;
    singleTagId?: Ref<Tag>;
    highlightedTagIds?: Ref<Tag>[];
  }
>(
  (
    {
      payload,
      dt,
      className,
      hideIndicator = false,
      hoveredTagId,
      maximumTags,
      singleTagId,
      highlightedTagIds,
    },
    ref,
  ) => {
    const involvedTagIds = useMemo(
      () => Object.keys(payload).map((id) => +id as Ref<Tag>),
      [payload],
    );
    const totalUsageTicks = useMemo(
      () => (singleTagId ? undefined : _(payload).values().sum()),
      [singleTagId, payload],
    );
    const involvedTags = useTags(involvedTagIds);
    const involvedTagSorted = useMemo(
      () =>
        _(involvedTags)
          .map((tag) => ({ tag, usageTicks: payload[tag.id]! }))
          .filter((v) => v.usageTicks > 0)
          .orderBy(["usageTicks"], ["desc"])
          .orderBy((v) => highlightedTagIds?.indexOf(v.tag.id) ?? -1, "desc")
          .value(),
      [involvedTags, payload, highlightedTagIds],
    );

    const highlightedTagId = singleTagId || hoveredTagId;
    const highlightedTag = involvedTags.find(
      (tag) => tag.id === highlightedTagId,
    );
    const highlightedTagUsageTicks = highlightedTagId
      ? (payload[highlightedTagId] ?? 0)
      : 0;

    return (
      <div
        className={cn("grid max-w-80 items-start gap-1.5 text-xs", className)}
        ref={ref}
      >
        {highlightedTag ? (
          <HoverCard
            tag={highlightedTag}
            usageTicks={highlightedTagUsageTicks}
            totalUsageTicks={totalUsageTicks}
            at={dt}
          />
        ) : (
          dt && (
            <div className="flex text-muted-foreground">
              <DateTimeText
                className="text-xs text-muted-foreground"
                datetime={dt}
              />
              <div className="flex-1 min-w-4"></div>
              <DurationText ticks={totalUsageTicks ?? 0} />
            </div>
          )
        )}
        {!singleTagId && Object.keys(payload).length > 0 && (
          <div className="grid gap-1.5 pt-1 border-t border-border">
            {involvedTagSorted
              .slice(0, maximumTags)
              .map(({ tag, usageTicks }) => {
                return (
                  <div
                    key={tag.id}
                    className={cn(
                      "flex w-full flex-wrap items-stretch gap-2 [&>svg]:h-2.5 [&>svg]:w-2.5 [&>svg]:text-muted-foreground",
                    )}
                  >
                    <>
                      {!hideIndicator && (
                        <TagIcon
                          style={{ color: tag.color }}
                          className="w-4 h-4 shrink-0"
                        />
                      )}
                      <div className="flex flex-1 justify-between items-center">
                        <div className="grid gap-1.5">
                          <Text
                            className={cn({
                              "text-muted-foreground":
                                !highlightedTagIds?.includes(tag.id),
                            })}
                          >
                            {tag.name}
                          </Text>
                        </div>
                        <DurationText
                          className="font-mono font-medium tabular-nums text-foreground ml-2 shrink-0"
                          ticks={usageTicks}
                        />
                      </div>
                    </>
                  </div>
                );
              })}
            {maximumTags !== undefined &&
              involvedTagSorted.length > maximumTags && (
                <span className="text-muted-foreground">
                  + {involvedTagSorted.length - maximumTags} more
                </span>
              )}
          </div>
        )}
      </div>
    );
  },
);
TagUsageChartTooltipContent.displayName = "TagUsageChartTooltipContent";

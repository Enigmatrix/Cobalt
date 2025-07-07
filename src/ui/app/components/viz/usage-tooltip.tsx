import AppIcon from "@/components/app/app-icon";
import { ScoreCircle } from "@/components/tag/score";
import {
  fullKeyToString,
  type FullKey,
  type FullKeyWithDuration,
} from "@/components/target-keys";
import { DurationText } from "@/components/time/duration-text";
import { DateTimeText } from "@/components/time/time-text";
import { Text } from "@/components/ui/text";
import type { App, Ref, Tag } from "@/lib/entities";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import _ from "lodash";
import { TagIcon } from "lucide-react";
import type { DateTime } from "luxon";
import React, { useMemo } from "react";

export function UsageTooltipContent({
  at,
  data,
  hovered,
  maximum = 10,
  highlightedApps,
  highlightedTags,
  totalDuration,
  className,
}: {
  at?: DateTime;
  data?: FullKeyWithDuration[];
  hovered?: FullKeyWithDuration;
  maximum: number;
  highlightedApps?: Record<Ref<App>, boolean>;
  highlightedTags?: Record<Ref<Tag>, boolean>;
  totalDuration?: number;
  className?: ClassValue;
}) {
  const hasAnyHighlighted = useMemo(() => {
    if (!highlightedApps && !highlightedTags) return false;
    return (
      Object.values(highlightedApps ?? {}).some((v) => v) ||
      Object.values(highlightedTags ?? {}).some((v) => v)
    );
  }, [highlightedApps, highlightedTags]);

  const fullData = useMemo(() => {
    return _(data)
      .orderBy("duration", "desc")
      .orderBy((kv) => {
        const isHighlighted =
          kv.key === "app"
            ? highlightedApps?.[kv.app.id]
            : highlightedTags?.[kv.tag.id];
        return isHighlighted ? 0 : 1;
      })
      .value();
  }, [data, highlightedApps, highlightedTags]);

  return (
    <div className={cn("max-w-80 flex flex-col text-xs gap-1.5", className)}>
      {hovered ? (
        <HoverDisplay
          hovered={hovered}
          duration={hovered.duration}
          totalDuration={totalDuration}
          at={at}
        />
      ) : (
        at && (
          <div className="flex text-muted-foreground">
            <DateTimeText
              className="text-xs text-muted-foreground"
              datetime={at}
            />
            <div className="flex-1 min-w-4" />
            <DurationText ticks={totalDuration ?? 0} />
          </div>
        )
      )}

      {fullData && fullData.length > 0 && (
        <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-2 pt-1.5 border-t border-border">
          {fullData.slice(0, maximum).map((d) => {
            return (
              <React.Fragment key={fullKeyToString(d)}>
                {d.key === "app" ? (
                  <AppRow
                    app={d.app}
                    isHighlighted={
                      highlightedApps?.[d.app.id] ?? !hasAnyHighlighted
                    }
                  />
                ) : (
                  <TagRow
                    tag={d.tag}
                    isHighlighted={
                      highlightedTags?.[d.tag.id] ?? !hasAnyHighlighted
                    }
                  />
                )}
                <DurationText
                  className="font-mono text-right font-medium tabular-nums text-muted-foreground shrink-0 min-w-fit"
                  ticks={d.duration}
                />
              </React.Fragment>
            );
          })}
        </div>
      )}

      {maximum !== undefined && fullData && fullData.length > maximum && (
        <div className="text-muted-foreground mt-1">
          + {fullData.length - maximum} more
        </div>
      )}
    </div>
  );
}

function AppRow({ app, isHighlighted }: { app: App; isHighlighted: boolean }) {
  return (
    <div className="flex items-center gap-2 min-w-0">
      <AppIcon appIcon={app.icon} className="w-4 h-4 shrink-0" />
      <Text
        className={cn("min-w-0", {
          "text-muted-foreground": !isHighlighted,
        })}
      >
        {app.name}
      </Text>
    </div>
  );
}

function TagRow({ tag, isHighlighted }: { tag: Tag; isHighlighted: boolean }) {
  return (
    <div className="flex items-center gap-2 min-w-0">
      <TagIcon className="w-4 h-4 shrink-0" style={{ color: tag.color }} />
      <Text
        className={cn("min-w-0", {
          "text-muted-foreground": !isHighlighted,
        })}
      >
        {tag.name}
      </Text>
    </div>
  );
}

function HoverDisplay({
  hovered,
  duration,
  totalDuration,
  at,
}: {
  hovered: FullKey;
  duration: number;
  totalDuration?: number;
  at?: DateTime;
}) {
  return (
    <div className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center py-2 w-full">
      {hovered.key === "app" ? (
        <AppDisplay app={hovered.app} at={at} />
      ) : (
        <TagDisplay tag={hovered.tag} at={at} />
      )}

      <div className="text-right text-muted-foreground shrink-0 min-w-0 overflow-visible">
        <DurationText
          className="font-semibold text-sm min-w-full"
          ticks={duration}
        />
        {totalDuration && (
          <div className="text-xs shrink-0 inline-flex items-center gap-1 min-w-0">
            <div>/</div>
            <DurationText ticks={totalDuration} />
          </div>
        )}
      </div>
    </div>
  );
}

function AppDisplay({
  app,
  className,
  at,
}: {
  app: App;
  className?: ClassValue;
  at?: DateTime;
}) {
  return (
    <div className={cn("flex items-center gap-2 ml-2", className)}>
      <AppIcon appIcon={app.icon} className="w-6 h-6 shrink-0 mr-1" />
      <div className="flex flex-col mr-2">
        <Text className="text-base max-w-52">{app.name}</Text>
        {at && (
          <DateTimeText
            className="text-xs text-muted-foreground"
            datetime={at}
          />
        )}
      </div>
    </div>
  );
}

function TagDisplay({
  tag,
  className,
  at,
}: {
  tag: Tag;
  className?: ClassValue;
  at?: DateTime;
}) {
  return (
    <div className={cn("flex items-center gap-2 ml-2", className)}>
      <TagIcon className="w-6 h-6 shrink-0 mr-1" style={{ color: tag.color }} />
      <div className="flex flex-col mr-2">
        <div className="flex items-center gap-2">
          <Text className="text-base max-w-52">{tag.name}</Text>
          <ScoreCircle score={tag.score} />
        </div>
        {at && (
          <DateTimeText
            className="text-xs text-muted-foreground"
            datetime={at}
          />
        )}
      </div>
    </div>
  );
}

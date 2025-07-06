import AppIcon from "@/components/app/app-icon";
import { ScoreCircle } from "@/components/tag/score";
import { DurationText } from "@/components/time/duration-text";
import { DateTimeText } from "@/components/time/time-text";
import { Text } from "@/components/ui/text";
import type { FullValue, Key } from "@/components/viz/usage-chart";
import { useApp, useTag } from "@/hooks/use-refresh";
import type { App, Ref, Tag } from "@/lib/entities";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import _ from "lodash";
import { TagIcon } from "lucide-react";
import type { DateTime } from "luxon";
import { useMemo } from "react";

export function UsageTooltipContent({
  at,
  data,
  hovered,
  maximum = 10,
  highlightedApps,
  highlightedTags,
  className,
}: {
  at?: DateTime;
  data?: FullValue<{ duration: number }>[];
  hovered?: Key;
  maximum: number;
  highlightedApps?: Record<Ref<App>, boolean>;
  highlightedTags?: Record<Ref<Tag>, boolean>;
  className?: ClassValue;
}) {
  const hasAnyHighlighted = useMemo(() => {
    if (!highlightedApps && !highlightedTags) return false;
    return (
      Object.values(highlightedApps ?? {}).some((v) => v) ||
      Object.values(highlightedTags ?? {}).some((v) => v)
    );
  }, [highlightedApps, highlightedTags]);

  const totalUsageTicks = useMemo(() => {
    return data?.reduce((acc, curr) => acc + curr.duration, 0) ?? 0;
  }, [data]);

  // TODO: This is a hack to get the duration of the hovered item.
  // We should probably store the duration in the hovered data.
  const hoveredDuration = useMemo(
    () =>
      hovered
        ? (data?.find(
            (d) =>
              d.key === hovered.key &&
              (d.key === "app"
                ? d.app.id === hovered.id
                : d.tag.id === hovered.id),
          )?.duration ?? 0)
        : 0,
    [data, hovered],
  );

  const fullData = useMemo(() => {
    return _(data)
      .orderBy("duration", "desc")
      .orderBy(
        (d) =>
          d.key === "app"
            ? (highlightedApps?.[d.app.id] ?? true)
            : (highlightedTags?.[d.tag.id] ?? true),
        "desc",
      )
      .value();
  }, [data, highlightedApps, highlightedTags]);

  return (
    <div className={cn("max-w-80 flex flex-col text-xs gap-1.5", className)}>
      {hovered ? (
        <HoverDisplay
          id={hovered}
          usageTicks={hoveredDuration}
          totalUsageTicks={totalUsageTicks}
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
            <DurationText ticks={totalUsageTicks} />
          </div>
        )
      )}

      {fullData && fullData.length > 0 && (
        <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-2 pt-1.5 border-t border-border">
          {fullData.slice(0, maximum).map((d) => {
            return (
              <>
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
              </>
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
  id,
  usageTicks,
  totalUsageTicks,
  at,
}: {
  id: Key;
  usageTicks: number;
  totalUsageTicks?: number;
  at?: DateTime;
}) {
  return (
    <div className="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center py-2 w-full">
      {id.key === "app" ? (
        <AppDisplay id={id.id} at={at} />
      ) : (
        <TagDisplay id={id.id} at={at} />
      )}

      <div className="text-right text-muted-foreground shrink-0 min-w-0 overflow-visible">
        <DurationText
          className="font-semibold text-sm min-w-full"
          ticks={usageTicks}
        />
        {totalUsageTicks && (
          <div className="text-xs shrink-0 inline-flex items-center gap-1 min-w-0">
            <div>/</div>
            <DurationText ticks={totalUsageTicks} />
          </div>
        )}
      </div>
    </div>
  );
}

function AppDisplay({
  id,
  className,
  at,
}: {
  id: Ref<App>;
  className?: ClassValue;
  at?: DateTime;
}) {
  const app = useApp(id);
  if (!app) return null;

  return (
    <div className={cn("flex items-center gap-2 ml-2", className)}>
      <AppIcon appIcon={app.icon} className="w-6 h-6 shrink-0 mr-1" />
      <div className="flex flex-col">
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
  id,
  className,
  at,
}: {
  id: Ref<Tag>;
  className?: ClassValue;
  at?: DateTime;
}) {
  const tag = useTag(id);
  if (!tag) return null;

  return (
    <div className={cn("flex items-center gap-2 ml-2", className)}>
      <TagIcon className="w-6 h-6 shrink-0 mr-1" style={{ color: tag.color }} />
      <div className="flex flex-col">
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

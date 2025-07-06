import AppIcon from "@/components/app/app-icon";
import { ScoreCircle } from "@/components/tag/score";
import { DurationText } from "@/components/time/duration-text";
import { DateTimeText } from "@/components/time/time-text";
import { Text } from "@/components/ui/text";
import type { FullValue, Key, Value } from "@/components/viz/usage-chart";
import { useApp, useTag } from "@/hooks/use-refresh";
import type { App, Ref, Tag } from "@/lib/entities";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { TagIcon } from "lucide-react";
import type { DateTime } from "luxon";

function UsageTooltipContent({
  at,
  data,
  hovered,
  maximum = 10,
  highlightedApps,
  highlightedTags,
}: {
  at?: DateTime;
  data?: Value<{ duration: number }>[];
  hovered?: Key;
  maximum: number;
  highlightedApps?: Record<Ref<App>, boolean>;
  highlightedTags?: Record<Ref<Tag>, boolean>;
}) {}

function HoverDisplay({
  key,
  usageTicks,
  totalUsageTicks,
  at,
}: {
  key: Key;
  usageTicks: number;
  totalUsageTicks?: number;
  at?: DateTime;
}) {
  return (
    <div className="flex items-center gap-2 py-2">
      {key.key === "app" ? (
        <AppDisplay id={key.id} at={at} />
      ) : (
        <TagDisplay id={key.id} at={at} />
      )}

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
    <div className={cn(className)}>
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
    <div className={cn(className)}>
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

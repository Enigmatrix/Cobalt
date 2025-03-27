import type { App, Ref, Tag } from "@/lib/entities";
import type { WithDuration } from "@/lib/entities";
import { untagged, type EntityMap } from "@/lib/state";
import type { ClassValue } from "clsx";
import { cn } from "@/lib/utils";
import { useApps, useTags } from "@/hooks/use-refresh";
import { useMemo, useRef, useState } from "react";
import {
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  type PieLabel,
} from "recharts";
import { toDataUrl } from "@/components/app/app-icon";
import _ from "lodash";
import { DurationText } from "@/components/time/duration-text";
import { AppUsageChartTooltipContent } from "@/components/viz/app-usage-chart-tooltip";
import { TagUsageChartTooltipContent } from "@/components/viz/tag-usage-chart-tooltip";
import { Tooltip } from "@/components/viz/tooltip";

const RADIAN = Math.PI / 180;

export interface AppUsagePieChartProps {
  data: EntityMap<App, WithDuration<App>>;
  // apps to highlight, order them by index in array. will be drawn left to right = top to bottom. unhighlighted apps will be drawn on top of highlighted apps.
  highlightedAppIds?: Ref<App>[];
  unhighlightedAppOpacity?: number;
  hideApps?: Record<Ref<App>, boolean>;

  className?: ClassValue;
  onHover?: (data?: WithDuration<App>) => void;
  onTagHover?: (data?: Tag) => void;
}

export function AppUsagePieChart({
  data,
  highlightedAppIds,
  unhighlightedAppOpacity = 0.5,
  hideApps = {},
  className,
  onHover,
  onTagHover,
}: AppUsagePieChartProps) {
  const appIds = useMemo(
    () => Object.keys(data).map((id) => +id as Ref<App>),
    [data],
  );
  const apps = useApps(appIds);
  const tags = useTags();

  const totalUsage = useMemo(() => {
    return _(apps)
      .filter((app) => !hideApps[app.id])
      .reduce((sum, app) => sum + (data[app.id]?.duration ?? 0), 0);
  }, [apps, data, hideApps]);

  const payload = useMemo(() => {
    return _.mapValues(data, (duration) => duration?.duration ?? 0);
  }, [data]);

  const tagPayload = useMemo(() => {
    return _(apps)
      .map((app) => [app.tagId ?? -1, data[app.id]?.duration ?? 0])
      .groupBy(0)
      .mapValues((x) => x.reduce((sum, [, duration]) => sum + duration, 0))
      .value();
  }, [apps, data]);

  const appData = useMemo(() => {
    return _(apps)
      .filter((app) => !hideApps[app.id])
      .map((app) => ({
        ...app,
        duration: data[app.id]?.duration ?? 0,
      }))
      .filter((app) => app.duration > 0)
      .orderBy(["tagId", "duration"], ["asc", "desc"])
      .value();
  }, [apps, data, hideApps]);

  const tagData = useMemo(() => {
    // Get all tagged apps
    const taggedAppIds = new Set(tags.flatMap((tag) => tag.apps));

    // Calculate untagged duration
    const untaggedDuration = apps
      .filter((app) => !taggedAppIds.has(app.id) && !hideApps[app.id])
      .reduce((sum, app) => sum + (data[app.id]?.duration ?? 0), 0);

    // Create tag data including untagged
    const tagData = [
      // Tags
      ...tags.map((tag) => ({
        ...tag,
        duration: _(tag.apps)
          .map((appId) => data[appId]?.duration ?? 0)
          .sum(),
      })),
      // Add untagged category if there are untagged apps
      ...(untaggedDuration > 0
        ? [
            {
              ...untagged,
              duration: untaggedDuration,
            },
          ]
        : []),
    ].filter((tag) => tag.duration > 0);

    return tagData;
  }, [tags, apps, data, hideApps]);

  const [hoveredApp, setHoveredApp] = useState<WithDuration<App> | undefined>(
    undefined,
  );
  const [hoveredTag, setHoveredTag] = useState<WithDuration<Tag> | undefined>(
    undefined,
  );

  const renderCustomizedLabel: PieLabel = (props) => {
    const { cx, cy, midAngle, innerRadius, outerRadius, percent, icon } = props;
    const url = toDataUrl(icon);
    const radius = innerRadius + (outerRadius - innerRadius) * 0.5;
    const x = cx + radius * Math.cos(-midAngle * RADIAN);
    const y = cy + radius * Math.sin(-midAngle * RADIAN);
    const radiusDiff = outerRadius - innerRadius;
    const angleLengthValue = percent * 360 * RADIAN * radius;
    const maxSize = Math.min(radiusDiff, angleLengthValue) * Math.SQRT1_2;
    const size = Math.max(Math.min(maxSize * 0.9, 32), 0); // 10% padding

    return (
      <image
        x={x - size / 2}
        y={y - size / 2}
        width={size}
        height={size}
        href={url}
        pointerEvents="none"
      />
    );
  };

  const containerRef = useRef<HTMLDivElement | null>(null);

  return (
    <div
      ref={containerRef}
      className={cn("w-full h-full min-h-[300px] relative", className)}
    >
      <Tooltip targetRef={containerRef} show={!!hoveredApp}>
        <AppUsageChartTooltipContent
          hoveredAppId={hoveredApp?.id ?? null}
          maximumApps={10}
          highlightedAppIds={highlightedAppIds}
          payload={payload}
        />
      </Tooltip>
      <Tooltip targetRef={containerRef} show={!!hoveredTag}>
        <TagUsageChartTooltipContent
          hoveredTagId={hoveredTag?.id ?? null}
          maximumTags={10}
          payload={tagPayload}
        />
      </Tooltip>
      <div className="absolute inset-0 flex items-center justify-center">
        <DurationText ticks={totalUsage} />
      </div>
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          {/* Inner pie chart for tags */}
          <Pie
            isAnimationActive={false}
            data={tagData}
            startAngle={0}
            cx="50%"
            cy="50%"
            innerRadius={60}
            outerRadius={80}
            dataKey="duration"
          >
            {tagData.map((entry) => (
              <Cell
                style={{
                  outline: "none",
                }}
                stroke="transparent"
                key={`tag-${entry.id}`}
                fill={entry.color}
                onMouseEnter={() => {
                  setHoveredTag(entry);
                  onTagHover?.(entry);
                }}
                onMouseLeave={() => {
                  setHoveredTag(undefined);
                  onTagHover?.(undefined);
                }}
              />
            ))}
          </Pie>

          {/* Outer pie chart for apps */}
          <Pie
            isAnimationActive={false}
            data={appData}
            startAngle={0}
            cx="50%"
            cy="50%"
            innerRadius={90}
            outerRadius={130}
            dataKey="duration"
            labelLine={false}
            label={renderCustomizedLabel}
          >
            {appData.map((entry) => (
              <Cell
                style={{
                  outline: "none",
                }}
                stroke="transparent"
                key={`app-${entry.id}`}
                fill={entry.color}
                opacity={
                  (highlightedAppIds?.includes(entry.id) ?? true)
                    ? 1
                    : unhighlightedAppOpacity
                }
                onMouseEnter={() => {
                  setHoveredApp(entry);
                  onHover?.(entry);
                }}
                onMouseLeave={() => {
                  setHoveredApp(undefined);
                  onHover?.(undefined);
                }}
              />
            ))}
          </Pie>
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
}

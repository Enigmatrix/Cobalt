import type { App, Ref, Tag } from "@/lib/entities";
import type { WithDuration } from "@/lib/entities";
import type { EntityMap } from "@/lib/state";
import type { ClassValue } from "clsx";
import { cn } from "@/lib/utils";
import { useApps, useTags } from "@/hooks/use-refresh";
import { useMemo } from "react";
import {
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  Tooltip,
  type PieLabel,
  type TooltipProps,
} from "recharts";
import { toDataUrl } from "@/components/app/app-icon";
import _ from "lodash";
import { DurationText } from "@/components/time/duration-text";

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

// TODO make this the same as the other untagged color we use
const UNTAGGED_COLOR = "#6B7280"; // gray-500

export function AppUsagePieChart({
  data,
  highlightedAppIds,
  unhighlightedAppOpacity = 0.5,
  hideApps = {},
  className,
  onHover,
  onTagHover,
}: AppUsagePieChartProps) {
  const apps = useApps(Object.keys(data).map((id) => +id as Ref<App>));
  const tags = useTags();

  const totalUsage = useMemo(() => {
    return _(apps)
      .filter((app) => !hideApps[app.id])
      .reduce((sum, app) => sum + (data[app.id]?.duration ?? 0), 0);
  }, [apps, data, hideApps]);

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
              id: -1 as Ref<Tag>,
              name: "Untagged",
              color: UNTAGGED_COLOR,
              apps: [],
              usages: { today: 0, week: 0, month: 0 },
              duration: untaggedDuration,
            },
          ]
        : []),
    ].filter((tag) => tag.duration > 0);

    return tagData;
  }, [tags, apps, data, hideApps]);

  // sussy argument
  const CustomTooltip = ({ active, payload }: TooltipProps<number, string>) => {
    if (active && payload && payload.length) {
      const data = payload[0].payload;
      return (
        <div className="bg-background border rounded-lg p-2 shadow-lg">
          <p className="font-medium">{data.name}</p>
          <p className="text-sm text-muted-foreground">
            Duration: {Math.round(data.duration / 1000)}s
          </p>
        </div>
      );
    }
    return null;
  };

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

  return (
    <div className={cn("w-full h-full min-h-[300px] relative", className)}>
      <div className="absolute inset-0 flex items-center justify-center">
        <DurationText ticks={totalUsage} />
      </div>
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          {/* Inner pie chart for tags */}
          <Pie
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
                key={`tag-${entry.id}`}
                fill={entry.color}
                onMouseEnter={() => onTagHover?.(entry)}
                onMouseLeave={() => onTagHover?.(undefined)}
              />
            ))}
          </Pie>

          {/* Outer pie chart for apps */}
          <Pie
            data={appData}
            startAngle={0}
            cx="50%"
            cy="50%"
            innerRadius={90}
            outerRadius={110}
            dataKey="duration"
            labelLine={false}
            label={renderCustomizedLabel}
          >
            {appData.map((entry) => (
              <Cell
                key={`app-${entry.id}`}
                fill={entry.color}
                opacity={
                  (highlightedAppIds?.includes(entry.id) ?? true)
                    ? 1
                    : unhighlightedAppOpacity
                }
                onMouseEnter={() => onHover?.(entry)}
                onMouseLeave={() => onHover?.(undefined)}
              />
            ))}
          </Pie>

          <Tooltip content={<CustomTooltip />} />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
}

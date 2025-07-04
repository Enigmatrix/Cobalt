import type { App, Ref, Tag, WithGroupedDuration } from "@/lib/entities";
import type { EntityMap } from "@/lib/state";
import type { Period } from "@/lib/time";
import type { ClassValue } from "clsx";
import type { DateTime, Duration } from "luxon";

interface UsageMarkerLine {
  yAxis: number;
  color?: string;
  type?: "dashed" | "solid";
}

type AppKey = { key: "app"; id: Ref<App> };
type TagKey = { key: "tag"; id: Ref<Tag> };
type Key = AppKey | TagKey;

interface UsageChartProps {
  className?: ClassValue;

  /// Data
  usages: EntityMap<App, WithGroupedDuration<App>[]>;
  start: DateTime;
  end: DateTime;
  period: Period;
  // Group apps by tag
  groupBy?: "tag" | "tag-show-untagged";

  /// Highlighting / Hiding
  highlightedAppIds?: Ref<App>[];
  unhighlightedAppOpacity?: number;
  // Apps/Tags to hide if true
  hidden?: Map<Key, boolean>;
  // Special mode to only show one app - doesn't show icon, for example
  onlyShowOneAppId?: Ref<App>;

  /// Chart Options
  animationsEnabled?: boolean;
  hideXAxis?: boolean;
  hideYAxis?: boolean;
  gradientBars?: boolean;
  barRadius?: number | [number, number, number, number];
  xAxisFormatter?: (dt: DateTime) => string;
  // Whether the Y-axis maximum should equal the period duration
  maxYIsPeriod?: boolean;
  // The interval between Y-axis ticks
  yAxisInterval?: Duration;

  /// Extra Lines to draw on Grid
  markerLines?: UsageMarkerLine[];

  /// Callbacks
  onHover?: (data?: WithGroupedDuration<Key>) => void;
}

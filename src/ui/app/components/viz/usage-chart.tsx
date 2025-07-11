import {
  appIconHtmlImgElement,
  tagIconUrlStatic,
} from "@/components/app/app-icon";
import {
  fullKeyToString,
  stringToFullKey,
  stringToKey,
  type AppFullKey,
  type FullKeyWith,
  type FullKeyWithDuration,
  type KeyWith,
} from "@/components/target-keys";
import { UsageTooltipContent } from "@/components/viz/usage-tooltip";
import { VizTooltip } from "@/components/viz/viz-tooltip";
import { useRefresh } from "@/hooks/use-refresh";
import { getVarColorAsHex, scaleColor } from "@/lib/color-utils";
import type { App, Ref, Tag, WithGroupedDuration } from "@/lib/entities";
import { untagged, useAppState, type EntityMap } from "@/lib/state";
import {
  dateTimeToTicks,
  durationToTicks,
  periodToDuration,
  ticksToDateTime,
  toHumanDateTime,
  toHumanDuration,
  type Period,
} from "@/lib/time";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import * as echarts from "echarts";
import _ from "lodash";
import { DateTime, type Duration } from "luxon";
import { useEffect, useMemo, useRef, useState } from "react";

interface UsageChartProps {
  className?: ClassValue;

  /// Data
  appDurationsPerPeriod: EntityMap<App, WithGroupedDuration<App>[]>;
  start: DateTime;
  end: DateTime;
  period: Period;
  // Group apps by tag
  groupBy?: GroupBy;

  /**
   * Highlighting / Hiding / Sorting.
   * In groupBy !== undefined,
   *   If a Tag has Apps in highlightedApps but it's not in highlightedTags, the tag will NOT be highlighted.
   *   If a Tag has Apps in hiddenApps but it's not in hiddenTags, the tag will be hidden.
   */
  /// Highlighting / Hiding
  highlightedApps?: Record<Ref<App>, boolean>; // If true, the app is highlighted
  highlightedTags?: Record<Ref<Tag>, boolean>; // If true, the tag is highlighted
  unhighlightedOpacity?: number;
  // Apps/Tags to hide if true
  hiddenApps?: Record<Ref<App>, boolean>; // If true, the app is hidden
  hiddenTags?: Record<Ref<Tag>, boolean>; // If true, the tag is hidden
  // Sorting
  sortedApps?: Ref<App>[];
  sortedTags?: Ref<Tag>[];
  // Special mode to only show one app/tag
  onlyShowOneApp?: Ref<App>;
  onlyShowOneTag?: Ref<Tag>;

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
  onHover?: (data?: KeyWith<{ group: number; duration: number }>) => void;
}

interface UsageMarkerLine {
  yAxis: number;
  color?: string;
  type?: "dashed" | "solid";
}

export type GroupBy = "app" | "tag" | "tag-show-untagged";

// group tick index, duration
type DataItem = [number, number];

interface HoverData {
  at: DateTime;
  hovered?: FullKeyWithDuration;
  data?: FullKeyWithDuration[];
}

// Get range of datetimes for the period, and convert to ticks
function getDateTimeRangePerPeriod(
  start: DateTime,
  end: DateTime,
  period: Period,
): number[] {
  const startRange = start.startOf(period);
  const range = [];
  for (let i = startRange; i < end; i = i.plus({ [period]: 1 })) {
    range.push(i);
  }
  return range.map((dt) => dateTimeToTicks(dt));
}

export function UsageChart({
  className,

  /// Data
  appDurationsPerPeriod,
  start,
  end,
  period,
  // Group apps by tag
  groupBy = "app",

  /// Highlighting / Hiding
  highlightedApps,
  highlightedTags,
  unhighlightedOpacity = 0.3,
  // Apps/Tags to hide if true
  hiddenApps,
  hiddenTags,
  // Sorting
  sortedApps,
  sortedTags,
  // Special mode to only show one app/tag
  onlyShowOneApp,
  onlyShowOneTag,

  /// Chart Options
  animationsEnabled = true,
  hideXAxis = false,
  hideYAxis = false,
  gradientBars = false,
  barRadius,
  xAxisFormatter = toHumanDateTime,
  // Whether the Y-axis maximum should equal the period duration
  maxYIsPeriod = false,
  // The interval between Y-axis ticks
  yAxisInterval,

  /// Extra Lines to draw on Grid
  markerLines,

  /// Callbacks
  onHover,
}: UsageChartProps) {
  // ECharts instance
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);

  // Hover
  const [hoveredData, setHoveredData] = useState<HoverData | null>(null);

  // App/Tag State
  const apps = useAppState((state) => state.apps);
  const tags = useAppState((state) => state.tags);

  const {
    xAxisValues,
    data: fullKeyValues,
    totalUsagePerPeriod,
  } = useUsageChartData({
    start,
    end,
    period,
    appDurationsPerPeriod,
    hiddenApps,
    hiddenTags,
    sortedApps,
    sortedTags,
    groupBy,
    apps,
    tags,
    onlyShowOneApp,
    onlyShowOneTag,
  });

  const hasAnyHighlighted = useMemo(() => {
    if (!highlightedApps && !highlightedTags) return false;
    return (
      Object.values(highlightedApps ?? {}).some((v) => v) ||
      Object.values(highlightedTags ?? {}).some((v) => v)
    );
  }, [highlightedApps, highlightedTags]);

  const fullKeyValuesWithHighlighted = useMemo(() => {
    return _(fullKeyValues)
      .orderBy((kv) => {
        const isHighlighted =
          kv.key === "app"
            ? highlightedApps?.[kv.app.id]
            : highlightedTags?.[kv.tag.id];
        return isHighlighted ? 0 : 1;
      })
      .value();
  }, [fullKeyValues, highlightedApps, highlightedTags]);

  const series = useMemo(() => {
    return fullKeyValuesWithHighlighted.map((kv) => {
      const id = fullKeyToString(kv);
      const name = kv.key === "app" ? kv.app.name : kv.tag.name;
      const color = kv.key === "app" ? kv.app.color : kv.tag.color;
      const isHighlighted =
        kv.key === "app"
          ? highlightedApps?.[kv.app.id]
          : highlightedTags?.[kv.tag.id];
      return {
        id,
        name,
        type: "bar",
        stack: "total",
        // [index, value][]
        data: kv.values,

        itemStyle: {
          opacity:
            (isHighlighted ?? !hasAnyHighlighted)
              ? undefined
              : unhighlightedOpacity,
          borderRadius: barRadius ?? 2,
          color: !gradientBars
            ? color
            : {
                type: "linear",
                x: 0,
                y: 0,
                x2: 0,
                y2: 1,
                colorStops: [
                  {
                    offset: 0,
                    color,
                  },
                  {
                    offset: 1,
                    color: scaleColor(color, 0.7),
                  },
                ],
              },
        },

        labelLayout(params) {
          let diam = Math.min(params.rect.width, params.rect.height) * 0.7;
          diam = Math.min(diam, 32);
          diam = diam < 5 ? 0 : diam;
          return { width: diam, height: diam };
        },

        label: {
          show: !onlyShowOneApp,
          position: "inside",
          backgroundColor: {
            image:
              kv.key === "app"
                ? appIconHtmlImgElement(kv.app.icon)
                : tagIconUrlStatic(
                    `color-mix(in srgb, ${kv.tag.color} 65%, black)`,
                  ),
          },
          formatter: () => {
            return `{empty|}`;
          },
          rich: {
            empty: {},
          },
        },
      } as echarts.SeriesOption;
    });
  }, [
    fullKeyValuesWithHighlighted,
    highlightedApps,
    highlightedTags,
    unhighlightedOpacity,
    barRadius,
    gradientBars,
    onlyShowOneApp,
    hasAnyHighlighted,
  ]);

  useEffect(() => {
    if (!chartRef.current) return;

    const chart = echarts.init(chartRef.current, undefined, {});
    chartInstanceRef.current = chart;

    // Period ticks
    const periodTicks = durationToTicks(periodToDuration(period));

    const option: echarts.EChartsOption = {
      animation: animationsEnabled,
      // animationDuration: 300,

      tooltip: {
        trigger: "axis",
        axisPointer: {
          type: "shadow",

          label: {
            show: false,
            formatter: (params) => {
              const at = ticksToDateTime(+params.value);
              const data: FullKeyWithDuration[] = params.seriesData.map(
                (series) => {
                  const duration = (series.data as DataItem)[1];
                  const key = stringToFullKey(series.seriesId!, apps, tags);
                  return { ...key, duration };
                },
              );
              setHoveredData((hoverData) => ({
                ...(hoverData ?? {}),
                at,
                data,
              }));
              return "";
            },
          },
        },
        formatter() {
          // disables echarts tooltip
          return "";
        },
      },
      grid: {
        left: "4px",
        right: "4px",
        bottom: hideXAxis ? "0" : "15px",
        top: hideYAxis ? "0" : "15px",
        containLabel: true,
      },
      xAxis: {
        type: "category",
        data: xAxisValues,
        axisLabel: {
          show: !hideXAxis,
          padding: [6, 0, 0, 0],
          formatter: (value: string) => {
            const dt = ticksToDateTime(+value);
            return xAxisFormatter(dt);
          },
        },
        show: !hideXAxis,
        axisLine: {
          show: !hideXAxis,
        },
        axisTick: {
          alignWithLabel: true,
          show: !hideXAxis,
        },
      },
      yAxis: {
        type: "value",
        show: !hideYAxis,
        min: 0,
        max: maxYIsPeriod ? periodTicks : undefined,
        interval: yAxisInterval
          ? durationToTicks(yAxisInterval)
          : maxYIsPeriod
            ? periodTicks / 4
            : undefined,
        splitLine: {
          show: !hideYAxis,

          lineStyle: {
            color: getVarColorAsHex("muted-foreground", 0.15),
          },
        },
        axisLine: {
          show: !hideYAxis,
        },
        axisLabel: {
          show: !hideYAxis,
          formatter: (params) => toHumanDuration(params),
        },
      },
      series: [
        ...series,
        // Marker Lines
        ...((markerLines ?? []).map((line) => ({
          type: "line",
          markLine: {
            silent: true,
            symbol: "none",
            data: [
              {
                yAxis: line.yAxis,
                lineStyle: {
                  color: line.color ?? getVarColorAsHex("muted-foreground"),
                  type: line.type,
                },
              },
            ],
          },
          data: [],
        })) satisfies echarts.SeriesOption[]),
      ],
    } satisfies echarts.EChartsOption;

    chart.getZr().on("mousemove", (params) => {
      const pos = [params.offsetX, params.offsetY];
      const isInGrid = chart.containPixel("grid", pos);
      if (isInGrid) {
        const pointerData = chart.convertFromPixel("grid", pos);
        setHoveredData({
          at: ticksToDateTime(xAxisValues[pointerData[0]]),
        });
      } else if (!isInGrid) {
        setHoveredData(null);
        onHover?.(undefined);
      }
    });

    chart.on("mousemove", (params) => {
      const ticks = +params.name;
      const at = ticksToDateTime(ticks);
      const duration = (params.data as DataItem)[1];
      const key = stringToFullKey(params.seriesId!, apps, tags);

      setHoveredData((hoverData) => ({
        ...hoverData,
        at,
        hovered: { ...key, duration },
      }));

      onHover?.({
        ...stringToKey(params.seriesId!),
        group: ticks,
        duration,
      });
    });

    const resizeObserver = new ResizeObserver(() => {
      requestAnimationFrame(() => chart.resize());
    });

    chart.on("finished", () => {
      resizeObserver.observe(chartRef.current!);
    });

    chart.setOption(option);

    return () => {
      chart.dispose();
      resizeObserver.disconnect();
    };
  }, [
    series,
    apps,
    tags,
    xAxisValues,
    animationsEnabled,
    hideXAxis,
    hideYAxis,
    maxYIsPeriod,
    yAxisInterval,
    markerLines,
    period,
    xAxisFormatter,
    onHover,
  ]);

  return (
    <div ref={chartRef} className={cn("w-full h-full", className)}>
      <VizTooltip targetRef={chartRef} show={!!hoveredData}>
        <UsageTooltipContent
          at={hoveredData?.at ?? undefined}
          data={hoveredData?.data ?? undefined}
          hovered={hoveredData?.hovered ?? undefined}
          maximum={10}
          highlightedApps={highlightedApps}
          highlightedTags={highlightedTags}
          totalDuration={
            totalUsagePerPeriod[dateTimeToTicks(hoveredData?.at ?? start)] ?? 0
          }
        />
      </VizTooltip>
    </div>
  );
}

function useUsageChartData({
  start,
  end,
  period,
  appDurationsPerPeriod,
  hiddenApps,
  hiddenTags,
  sortedApps,
  sortedTags,
  groupBy,
  apps,
  tags,
  onlyShowOneApp,
  onlyShowOneTag,
}: {
  start: DateTime;
  end: DateTime;
  period: Period;
  appDurationsPerPeriod: EntityMap<App, WithGroupedDuration<App>[]>;
  hiddenApps?: Record<Ref<App>, boolean>;
  hiddenTags?: Record<Ref<Tag>, boolean>;
  sortedApps?: Ref<App>[];
  sortedTags?: Ref<Tag>[];
  groupBy: GroupBy;
  apps: EntityMap<App, App>;
  tags: EntityMap<Tag, Tag>;
  onlyShowOneApp?: Ref<App>;
  onlyShowOneTag?: Ref<Tag>;
}) {
  const { handleStaleApps, handleStaleTags } = useRefresh();

  // Get the x-axis values and a lookup from datetime tick to index
  const [xAxisValues, xAxisTickToIndexLookup] = useMemo(() => {
    const xAxisValues = getDateTimeRangePerPeriod(start, end, period);
    const xAxisTickToIndexLookup = Object.fromEntries(
      xAxisValues.map((tick, index) => [tick, index]),
    );
    return [xAxisValues, xAxisTickToIndexLookup];
  }, [start, end, period]);

  const fullKeyValues = useMemo(() => {
    const appKeys = _(
      // If only showing one app, return the app
      onlyShowOneApp ? [onlyShowOneApp] : Object.keys(appDurationsPerPeriod),
    )
      // Map to apps
      .map((id) => apps[+id as Ref<App>])
      .thru(handleStaleApps)
      // Remove hidden apps
      .filter((app) => !hiddenApps?.[app.id])
      // Remove hidden tags
      .filter((app) => !hiddenTags?.[app.tagId ?? untagged.id])
      // Remove if not only showing one tag
      .filter(
        (app) =>
          !onlyShowOneTag || (app.tagId ?? untagged.id) === onlyShowOneTag,
      )

      .map((app) => ({ key: "app", app }) as AppFullKey)
      // Sort apps
      .orderBy((key) => sortedApps?.indexOf(key.app.id) ?? -1, "desc")
      .map((key) => ({
        ...key,
        values:
          appDurationsPerPeriod[key.app.id]?.map(
            (usage) =>
              [xAxisTickToIndexLookup[usage.group], usage.duration] as DataItem,
          ) ?? [],
      }));

    // Default grouping is none - just return the apps
    if (groupBy === "app") {
      return appKeys.value();
    }

    return (
      appKeys
        .groupBy((key) => key.app.tagId)
        .mapValues((apps, tagIdStr): FullKeyWith<{ values: DataItem[] }>[] => {
          const tagId = tagIdStr === "null" ? null : (+tagIdStr as Ref<Tag>);

          // If we're showing untagged as apps, return the apps
          if (tagId === null && groupBy === "tag-show-untagged") {
            return apps;
          }

          // Group usages by group for all apps in the tag, then sum the durations
          const values = _(apps)
            .map("values")
            .flatten()
            .groupBy(([group]) => group)
            .map(
              (durations, group) =>
                [
                  +group,
                  durations.reduce((acc, [, duration]) => acc + duration, 0),
                ] as DataItem,
            )
            .value();

          const tag = tagId ? tags[tagId] : untagged;
          return (
            _([tag])
              .thru(handleStaleTags)
              // Remove hidden tags
              .filter((tag) => !hiddenTags?.[tag.id])
              .map((tag) => ({ key: "tag" as const, tag, values }))
              .value()
          );
        })
        .values()
        .flatten()
        // Sort tags. If the key is an app, we place it at the end, but don't change the sort order of the apps (stable sort)
        .orderBy(
          (key) =>
            key.key === "tag" ? (sortedTags?.indexOf(key.tag.id) ?? -1) : -1,
          "desc",
        )
        .value()
    );
  }, [
    appDurationsPerPeriod,
    hiddenApps,
    hiddenTags,
    sortedApps,
    sortedTags,
    groupBy,
    apps,
    tags,
    handleStaleApps,
    handleStaleTags,
    xAxisTickToIndexLookup,
    onlyShowOneApp,
    onlyShowOneTag,
  ]);

  // map between group and total duration
  const totalUsagePerPeriod: Record<number, number> = useMemo(() => {
    return _(appDurationsPerPeriod)
      .flatMap((appDurations) => appDurations)
      .groupBy((appDur) => appDur?.group)
      .map(
        (durations, group) =>
          [
            +group,
            durations.reduce((acc, appDur) => acc + (appDur?.duration ?? 0), 0),
          ] as const,
      )
      .fromPairs()
      .value();
  }, [appDurationsPerPeriod]);

  return {
    xAxisValues,
    xAxisTickToIndexLookup,
    data: fullKeyValues,
    totalUsagePerPeriod,
  };
}

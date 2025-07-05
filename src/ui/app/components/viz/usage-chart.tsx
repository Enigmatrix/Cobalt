import {
  htmlImgElement,
  TAG_ICON_URL,
  tagIconUrlStatic,
} from "@/components/app/app-icon";
import { useRefresh } from "@/hooks/use-refresh";
import { getVarColorAsHex, scaleColor } from "@/lib/color-utils";
import type { App, Ref, Tag, WithGroupedDuration } from "@/lib/entities";
import { untagged, useAppState, type EntityMap } from "@/lib/state";
import {
  dateTimeToTicks,
  durationToTicks,
  periodToDuration,
  ticksToDateTime,
  toHumanDuration,
  type Period,
} from "@/lib/time";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import * as echarts from "echarts";
import _ from "lodash";
import type { DateTime, Duration } from "luxon";
import { useEffect, useMemo, useRef } from "react";

interface UsageChartProps {
  className?: ClassValue;

  /// Data
  usages: EntityMap<App, WithGroupedDuration<App>[]>;
  start: DateTime;
  end: DateTime;
  period: Period;
  // Group apps by tag
  groupBy?: "tag" | "tag-show-untagged";

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
  // Special mode to only show one app - doesn't show icon, and the tooltip is different
  onlyShowOneAppId?: boolean;

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
  onHover?: (data?: WithGroupedDuration<FullKey>) => void;
}

interface UsageMarkerLine {
  yAxis: number;
  color?: string;
  type?: "dashed" | "solid";
}

interface AppFullKey {
  key: "app";
  app: App;
}
interface TagFullKey {
  key: "tag";
  tag: Tag;
}
type FullKey = AppFullKey | TagFullKey;

type FullValues<T> = FullKey & { values: T };

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
  usages,
  start,
  end,
  period,
  // Group apps by tag
  groupBy,

  /// Highlighting / Hiding
  highlightedApps,
  highlightedTags,
  unhighlightedOpacity,
  // Apps/Tags to hide if true
  hiddenApps,
  hiddenTags,
  // Sorting
  sortedApps,
  sortedTags,
  // Special mode to only show one app - doesn't show icon, and the tooltip is different
  onlyShowOneAppId,

  /// Chart Options
  animationsEnabled,
  hideXAxis,
  hideYAxis,
  gradientBars,
  barRadius,
  xAxisFormatter,
  // Whether the Y-axis maximum should equal the period duration
  maxYIsPeriod,
  // The interval between Y-axis ticks
  yAxisInterval,

  /// Extra Lines to draw on Grid
  markerLines,

  /// Callbacks
  onHover,
}: UsageChartProps) {
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);

  const apps = useAppState((state) => state.apps);
  const tags = useAppState((state) => state.tags);
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
    const appKeys = _(Object.keys(usages))
      // Map to apps
      .map((id) => apps[id as unknown as Ref<App>])
      .thru(handleStaleApps)
      // Remove hidden apps
      .filter((app) => !hiddenApps?.[app.id])
      .map((app) => ({ key: "app", app }) as AppFullKey)
      // Sort apps
      .orderBy((key) => sortedApps?.indexOf(key.app.id) ?? -1, "desc")
      .map((key) => ({
        ...key,
        values: usages[key.app.id]!.map(
          (usage) =>
            [xAxisTickToIndexLookup[usage.group], usage.duration] as const,
        ),
      }));

    // Default grouping is none - just return the apps
    if (!groupBy) {
      return appKeys.value();
    }

    return (
      appKeys
        .groupBy((key) => key.app.tagId)
        .mapValues(
          (apps, tagIdStr): FullValues<(readonly [number, number])[]>[] => {
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
                  ] as const,
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
          },
        )
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
    usages,
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
  ]);
  
  const fullKeyValuesWithHighlighted = useMemo(() => {
    return _(fullKeyValues).orderBy((kv) => {
      const isHighlighted =
        kv.key === "app"
          ? highlightedApps?.[kv.app.id]
          : highlightedTags?.[kv.tag.id];
      return isHighlighted ? 0 : 1;
    }).value();
  }, [fullKeyValues, highlightedApps, highlightedTags]);

  const series = useMemo(() => {
    return fullKeyValuesWithHighlighted.map((kv) => {
      const id = kv.key === "app" ? "app-" + kv.app.id : "tag-" + kv.tag.id;
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
          opacity: (isHighlighted ?? true) ? undefined : unhighlightedOpacity,
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
          show: !onlyShowOneAppId,
          position: "inside",
          backgroundColor: {
            image:
              kv.key === "app"
                ? htmlImgElement(kv.app.icon)
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
    onlyShowOneAppId,
  ]);

  useEffect(() => {
    if (!chartRef.current) return;

    const chart = echarts.init(chartRef.current, undefined, {});
    chartInstanceRef.current = chart;

    // X-axis Gap
    const periodTicks = durationToTicks(periodToDuration(period));

    const option: echarts.EChartsOption = {
      animation: animationsEnabled,
      // animationDuration: 300,

      tooltip: {
        trigger: "axis",
        axisPointer: {
          type: "shadow",

          // TODO: Hover
          // label: {
          //   show: false,
          //   formatter: (params) => {
          //     const seriesValues: EntityMap<App, number> = Object.fromEntries(
          //       params.seriesData.map(
          //         (v) =>
          //           [+v.seriesId!, (v.value as [number, number])[1]] as const,
          //       ),
          //     );
          //     setHoverSeries(seriesValues);
          //     return "";
          //   },
          // },
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
            return xAxisFormatter?.(dt) ?? dt.toString();
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

    // TODO: Hover

    // chart.getZr().on("mousemove", (params) => {
    //   const pos = [params.offsetX, params.offsetY];
    //   const isInGrid = chart.containPixel("grid", pos);
    //   if (isInGrid) {
    //     const pointerData = chart.convertFromPixel("grid", pos);
    //     setHoveredData({
    //       date: ticksToDateTime(xaxisRange[pointerData[0]]),
    //     });
    //   } else if (!isInGrid) {
    //     setHoveredData(null);
    //     onHover?.(undefined);
    //   }
    // });

    // TODO: Hover

    // chart.on("mousemove", (params) => {
    //   const appId = +(params.seriesId ?? 0) as Ref<App>;
    //   setHoveredData({ date: ticksToDateTime(+params.name), appId });
    //   if (onHover) {
    //     onHover({
    //       id: appId,
    //       duration: params.value as number,
    //       group: params.axisValue as number,
    //     });
    //   }
    // });

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
      {/* TODO: Hover 
      <Tooltip targetRef={chartRef} show={!!hoveredData}>
        <AppUsageChartTooltipContent
          hoveredAppId={hoveredData?.appId ?? null}
          singleAppId={singleAppId}
          payload={hoverSeries}
          dt={hoveredData?.date ?? DateTime.fromSeconds(0)}
          maximumApps={10}
          highlightedAppIds={highlightedAppIds}
        />
      </Tooltip> */}
    </div>
  );
}

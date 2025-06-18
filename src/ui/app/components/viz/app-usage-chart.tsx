import React, { useMemo, useRef, useState } from "react";
import * as echarts from "echarts";
import { DateTime, Duration } from "luxon";
import _ from "lodash";
import { useAppState, type EntityMap } from "@/lib/state";
import { useRefresh } from "@/hooks/use-refresh";
import {
  type Period,
  dateTimeToTicks,
  durationToTicks,
  periodToDuration,
  ticksToDateTime,
  toHumanDateTime,
  toHumanDuration,
} from "@/lib/time";
import { htmlImgElement } from "@/components/app/app-icon";
import type { App, Ref, WithGroupedDuration } from "@/lib/entities";
import type { ClassValue } from "clsx";
import { cn } from "@/lib/utils";
import { Tooltip } from "@/components/viz/tooltip";
import { AppUsageChartTooltipContent } from "@/components/viz/app-usage-chart-tooltip";
import { useTheme } from "@/components/theme-provider";
import { getVarColorAsHex } from "@/lib/color-utils";

export interface AppUsageBarChartProps {
  data: EntityMap<App, WithGroupedDuration<App>[]>;
  // apps to highlight, order them by index in array. will be drawn left to right = top to bottom. unhighlighted apps will be drawn on top of highlighted apps.
  highlightedAppIds?: Ref<App>[];
  unhighlightedAppOpacity?: number;
  markerLines?: {
    yAxis: number;
    color?: string;
    type?: "dashed" | "solid";
  }[];
  hideApps?: Record<Ref<App>, boolean>;
  singleAppId?: Ref<App>;
  start: DateTime;
  end: DateTime;
  interval?: Duration;
  period: Period;

  maxYIsPeriod?: boolean;
  hideXAxis?: boolean;
  hideYAxis?: boolean;
  gridVertical?: boolean;
  gridHorizontal?: boolean;
  gradientBars?: boolean;
  animationsEnabled?: boolean;
  barRadius?: number | [number, number, number, number];
  className?: ClassValue;
  dateTimeFormatter?: (dt: DateTime) => string;
  onHover?: (data?: WithGroupedDuration<App>) => void;
}

function getDateTimeRange(
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

export function AppUsageBarChart({
  data,
  highlightedAppIds,
  unhighlightedAppOpacity = 0.3,
  markerLines,
  hideApps,
  singleAppId,
  start,
  end,
  maxYIsPeriod = false,
  interval,
  hideXAxis = false,
  hideYAxis = false,
  gradientBars = false,
  dateTimeFormatter = toHumanDateTime,
  period,
  animationsEnabled = true,
  className,
  onHover,
  barRadius,
}: AppUsageBarChartProps) {
  const { theme } = useTheme();
  const apps = useAppState((state) => state.apps);
  const { handleStaleApps } = useRefresh();
  const [hoverSeries, setHoverSeries] = useState<EntityMap<App, number>>({});
  const [hoveredData, setHoveredData] = useState<{
    date: DateTime;
    appId?: Ref<App>;
  } | null>(null);
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);

  const involvedApps = useMemo(
    () =>
      _(singleAppId ? [singleAppId] : Object.keys(data))
        .map((id) => apps[id as unknown as Ref<App>])
        .thru(handleStaleApps)
        .filter((app) => !hideApps?.[app.id])
        .orderBy((app) => highlightedAppIds?.indexOf(app.id) ?? -1, "desc")
        .value(),
    [handleStaleApps, apps, data, singleAppId, hideApps, highlightedAppIds],
  );

  React.useEffect(() => {
    if (!chartRef.current) return;

    const chart = echarts.init(chartRef.current, undefined, {});
    chartInstanceRef.current = chart;

    const xaxisRange = getDateTimeRange(start, end, period);
    const xaxisLookup = Object.fromEntries(
      xaxisRange.map((tick, index) => [tick, index]),
    );
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
              const seriesValues: EntityMap<App, number> = Object.fromEntries(
                params.seriesData.map(
                  (v) =>
                    [+v.seriesId!, (v.value as [number, number])[1]] as const,
                ),
              );
              setHoverSeries(seriesValues);
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
        data: xaxisRange,
        axisLabel: {
          show: !hideXAxis,
          padding: [6, 0, 0, 0],
          formatter: (value: string) =>
            dateTimeFormatter(ticksToDateTime(+value)),
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
        interval: interval
          ? durationToTicks(interval)
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
        ...(involvedApps.map((app) => ({
          id: app.id,
          name: app.name,
          type: "bar",
          stack: "total",
          // [index, value][]
          data: data[app.id]?.map((d) => [xaxisLookup[d.group], d.duration]),

          itemStyle: {
            opacity:
              (highlightedAppIds?.includes(app.id) ?? true)
                ? undefined
                : unhighlightedAppOpacity,
            color: gradientBars
              ? {
                  type: "linear",
                  x: 0,
                  y: 0,
                  x2: 0,
                  y2: 1,
                  colorStops: [
                    {
                      offset: 0,
                      color: app.color,
                    },
                    {
                      offset: 1,
                      color: echarts.color.modifyAlpha(app.color, 0.7),
                    },
                  ],
                }
              : app.color,
            borderRadius: barRadius ?? 2,
          },

          labelLayout(params) {
            let diam = Math.min(params.rect.width, params.rect.height) * 0.7;
            diam = Math.min(diam, 32);
            diam = diam < 5 ? 0 : diam;
            return { width: diam, height: diam };
          },
          label: {
            show: !singleAppId,
            position: "inside",
            backgroundColor: {
              image: htmlImgElement(app.id),
            },
            formatter: () => {
              return `{empty|}`;
            },
            rich: {
              empty: {},
            },
          },
        })) satisfies echarts.SeriesOption[]),
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
          date: ticksToDateTime(xaxisRange[pointerData[0]]),
        });
      } else if (!isInGrid) {
        setHoveredData(null);
        onHover?.(undefined);
      }
    });

    chart.on("mousemove", (params) => {
      const appId = +(params.seriesId ?? 0) as Ref<App>;
      setHoveredData({ date: ticksToDateTime(+params.name), appId });
      if (onHover) {
        onHover({
          id: appId,
          duration: params.value as number,
          group: params.axisValue as number,
        });
      }
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
    data,
    involvedApps,
    dateTimeFormatter,
    period,
    hideXAxis,
    hideYAxis,
    maxYIsPeriod,
    start,
    end,
    interval,
    singleAppId,
    gradientBars,
    barRadius,
    onHover,
    animationsEnabled,
    highlightedAppIds,
    unhighlightedAppOpacity,
    markerLines,
    theme,
  ]);

  return (
    <div ref={chartRef} className={cn("w-full h-full", className)}>
      <Tooltip targetRef={chartRef} show={!!hoveredData}>
        <AppUsageChartTooltipContent
          hoveredAppId={hoveredData?.appId ?? null}
          singleAppId={singleAppId}
          payload={hoverSeries}
          dt={hoveredData?.date ?? DateTime.fromSeconds(0)}
          maximumApps={10}
          highlightedAppIds={highlightedAppIds}
        />
      </Tooltip>
    </div>
  );
}

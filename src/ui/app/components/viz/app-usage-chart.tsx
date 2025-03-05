import React, { useMemo, useRef, useState } from "react";
import * as echarts from "echarts";
import { DateTime } from "luxon";
import _ from "lodash";
import { useAppState, type EntityMap } from "@/lib/state";
import { useRefresh } from "@/hooks/use-refresh";
import { ticksToDateTime, toHumanDateTime, toHumanDuration } from "@/lib/time";
import { toDataUrl } from "@/components/app/app-icon";
import type { App, Ref, WithGroupedDuration } from "@/lib/entities";
import type { ClassValue } from "clsx";
import { cn } from "@/lib/utils";
import { Tooltip } from "@/components/viz/tooltip";
import { AppUsageChartTooltipContent } from "@/components/viz/app-usage-chart-tooltip";

export interface AppUsageBarChartProps {
  data: EntityMap<App, WithGroupedDuration<App>[]>;
  singleAppId?: Ref<App>;
  periodTicks: number;
  rangeMinTicks: number;
  rangeMaxTicks: number;
  maxYIsPeriod?: boolean;
  hideXAxis?: boolean;
  hideYAxis?: boolean;
  gridVertical?: boolean;
  gridHorizontal?: boolean;
  gradientBars?: boolean;
  animationsEnabled?: boolean;
  className?: ClassValue;
  dateTimeFormatter?: (dt: DateTime) => string;
  onHover?: (data?: WithGroupedDuration<App>) => void;
  barRadius?: number | [number, number, number, number];
}

export function AppUsageBarChart({
  data,
  singleAppId,
  periodTicks,
  rangeMinTicks,
  rangeMaxTicks,
  maxYIsPeriod = false,
  hideXAxis = false,
  hideYAxis = false,
  gradientBars = false,
  dateTimeFormatter = toHumanDateTime,
  animationsEnabled = true,
  className,
  onHover,
  barRadius,
}: AppUsageBarChartProps) {
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
        .value(),
    [handleStaleApps, apps, data, singleAppId],
  );

  React.useEffect(() => {
    if (!chartRef.current) return;

    const chart = echarts.init(chartRef.current, undefined, {});
    chartInstanceRef.current = chart;

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
              const seriesValues = Object.fromEntries(
                params.seriesData.map((v) => [
                  v.seriesId,
                  (v.value as [number, number])[1],
                ]),
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
        data: _.range(rangeMinTicks!, rangeMaxTicks!, periodTicks),
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
        interval: periodTicks / 4,
        splitLine: {
          show: !hideYAxis,
          lineStyle: {
            color: "hsl(var(--muted))",
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
      series: involvedApps.map((app) => ({
        id: app.id,
        name: app.name,
        type: "bar",
        stack: "total",
        // [index, value][]
        data: data[app.id]?.map((d) => [
          (d.group - rangeMinTicks!) / periodTicks,
          d.duration,
        ]),

        itemStyle: {
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
            image: toDataUrl(app.icon)!,
          },
          formatter: () => {
            return `{empty|}`;
          },
          rich: {
            empty: {},
          },
        },
      })),
    } satisfies echarts.EChartsOption;

    chart.getZr().on("mousemove", (params) => {
      const pos = [params.offsetX, params.offsetY];
      const isInGrid = chart.containPixel("grid", pos);
      if (isInGrid) {
        const pointerData = chart.convertFromPixel("grid", pos);
        setHoveredData({
          date: ticksToDateTime(pointerData[0] * periodTicks + rangeMinTicks),
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
    hideXAxis,
    hideYAxis,
    maxYIsPeriod,
    periodTicks,
    rangeMinTicks,
    rangeMaxTicks,
    singleAppId,
    gradientBars,
    barRadius,
    onHover,
    animationsEnabled,
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
        />
      </Tooltip>
    </div>
  );
}

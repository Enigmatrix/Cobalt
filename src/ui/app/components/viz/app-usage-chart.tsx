import React, { useMemo, useRef, useState } from "react";
import * as echarts from "echarts";
import { DateTime } from "luxon";
import _ from "lodash";
import { useAppState, type EntityMap } from "@/lib/state";
import { useRefresh } from "@/hooks/use-refresh";
import { ticksToDateTime, toHumanDateTime } from "@/lib/time";
import { toDataUrl } from "@/components/app/app-icon";
import type { App, Ref, WithGroupedDuration } from "@/lib/entities";
import type { ClassValue } from "clsx";
import { cn } from "@/lib/utils";

export interface AppUsageBarChartProps {
  data: EntityMap<App, WithGroupedDuration<App>[]>;
  singleAppId?: Ref<App>;
  periodTicks: number;
  rangeMinTicks?: number;
  rangeMaxTicks?: number;
  maxYIsPeriod?: boolean;
  hideXAxis?: boolean;
  gridVertical?: boolean;
  gridHorizontal?: boolean;
  gradientBars?: boolean;
  animationsEnabled?: boolean;
  className?: ClassValue;
  dateTimeFormatter?: (dt: DateTime) => string;
  onHover?: (data?: WithGroupedDuration<App>) => void;
  barRadius?: number | [number, number, number, number];
}

type AppUsageBarChartData = {
  [app: Ref<App>]: number; // app => duration
  key: number; // group (timestamp)
};

export function AppUsageBarChart({
  data: unflattenedData,
  singleAppId,
  periodTicks,
  rangeMinTicks,
  rangeMaxTicks,
  maxYIsPeriod = false,
  hideXAxis = false,
  gridVertical = false,
  gridHorizontal = false,
  gradientBars = false,
  dateTimeFormatter = toHumanDateTime,
  animationsEnabled = true,
  className,
  onHover,
  barRadius,
}: AppUsageBarChartProps) {
  const apps = useAppState((state) => state.apps);
  const { handleStaleApps } = useRefresh();
  const [hoveredAppId, setHoveredAppId] = useState<Ref<App> | null>(null);
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);

  const involvedApps = useMemo(
    () =>
      _(
        singleAppId
          ? { [singleAppId]: unflattenedData[singleAppId] }
          : unflattenedData,
      )
        .keys()
        .map((id) => apps[id as unknown as Ref<App>])
        .thru(handleStaleApps)
        .value(),
    [handleStaleApps, apps, unflattenedData, singleAppId],
  );

  const data: AppUsageBarChartData[] = useMemo(() => {
    let ret = _(unflattenedData)
      .values()
      .thru(handleStaleApps)
      .flatten()
      .groupBy((d) => d.group)
      .mapValues((durs) => {
        return _.fromPairs([
          ...durs.map((d) => {
            return [d.id, d.duration];
          }),
          ["key", ticksToDateTime(durs[0].group).toMillis()],
        ]);
      })
      .value();

    if (rangeMinTicks !== undefined && rangeMaxTicks !== undefined) {
      ret = _.merge(
        ret,
        _(_.range(rangeMinTicks, rangeMaxTicks, periodTicks))
          .map((t) => {
            return [t, { key: ticksToDateTime(t).toMillis() }];
          })
          .fromPairs()
          .value(),
      );
    }

    return _(ret)
      .values()
      .flatten()
      .sortBy((d) => d.key)
      .value();
  }, [
    unflattenedData,
    handleStaleApps,
    rangeMinTicks,
    rangeMaxTicks,
    periodTicks,
  ]);

  React.useEffect(() => {
    if (!chartRef.current) return;

    const chart = echarts.init(chartRef.current, undefined, {});
    chartInstanceRef.current = chart;

    const option: echarts.EChartsOption = {
      animation: animationsEnabled,
      tooltip: {
        trigger: "axis",
        axisPointer: {
          type: "shadow",
        },
      },
      grid: {
        left: "5%",
        right: "5%",
        top: 0,
        bottom: hideXAxis ? "0" : "25px",
        containLabel: false,
      },
      xAxis: {
        type: "category",
        data: data.map((d) => d.key),
        axisLabel: {
          formatter: (value: string) =>
            dateTimeFormatter(DateTime.fromMillis(+value)),
        },
        show: !hideXAxis,
        axisLine: {
          show: !hideXAxis,
        },
        axisTick: {
          alignWithLabel: true,
          show: !hideXAxis,
        },
        splitLine: {
          show: gridVertical,
        },
      },
      yAxis: {
        type: "value",
        max: maxYIsPeriod ? periodTicks : undefined,
        show: false,
      },
      series: involvedApps.map((app) => ({
        id: app.id,
        name: app.name,
        type: "bar",
        stack: "total",
        data: data.map((d) => d[app.id] || 0),
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
          const diam = _.clamp(
            0,
            Math.min(params.rect.width, params.rect.height) * 0.7,
            32,
          );
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

    chart.setOption(option);

    chart.on("mouseover", (params) => {
      setHoveredAppId(params.id as Ref<App>);
      if (onHover) {
        onHover({
          id: params.id as Ref<App>,
          duration: params.value as number,
          group: params.axisValue as number,
        });
      }
    });

    chart.on("mouseout", () => {
      setHoveredAppId(null);
      if (onHover) {
        onHover(undefined);
      }
    });

    const resizeObserver = new ResizeObserver(() => {
      chart.resize();
    });

    resizeObserver.observe(chartRef.current);

    return () => {
      chart.dispose();
      resizeObserver.disconnect();
    };
  }, [
    data,
    involvedApps,
    dateTimeFormatter,
    hideXAxis,
    maxYIsPeriod,
    periodTicks,
    singleAppId,
    gradientBars,
    barRadius,
    onHover,
    animationsEnabled,
    gridVertical,
  ]);

  return <div ref={chartRef} className={cn("w-full h-full", className)} />;
}

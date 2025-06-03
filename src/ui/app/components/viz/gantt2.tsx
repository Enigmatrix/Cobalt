import { useEffect, useMemo, useRef } from "react";
import * as echarts from "echarts";
import type { App, Ref, Usage } from "@/lib/entities";
import type { InteractionPeriod, SystemEvent } from "@/lib/entities";
import type { AppSessionUsages } from "@/lib/repo";
import { ticksToDateTime, type Interval } from "@/lib/time";
import { useWidth } from "@/hooks/use-width";
import _ from "lodash";

type RectLike = {
  x: number;
  y: number;
  width: number;
  height: number;
};

const minRenderWidth = 1;

function minRenderTimeGap(interval: Interval, width: number, dataZoom: number) {
  const timeGap = interval.end.diff(interval.start).toMillis();
  const zoom = dataZoom / 100;
  const minRenderTimeGapMillis = ((timeGap * zoom) / width) * minRenderWidth;
  const minRenderTimeGap = minRenderTimeGapMillis * 10_000;
  return Math.max(minRenderTimeGap, 1);
}

function mergedUsages(usages: Usage[], minRenderTimeGap: number) {
  usages.sort((a, b) => a.start - b.start);
  const mergedUsages = [usages[0]];
  for (const usage of usages) {
    const lastUsage = mergedUsages[mergedUsages.length - 1];
    if (lastUsage.end + minRenderTimeGap >= usage.start) {
      lastUsage.end = usage.end;
    } else {
      mergedUsages.push(usage);
    }
  }
  return mergedUsages;
}

interface GanttProps {
  usages: AppSessionUsages;
  usagesLoading?: boolean;

  interactionPeriods?: InteractionPeriod[];
  interactionPeriodsLoading?: boolean;

  systemEvents?: SystemEvent[];
  systemEventsLoading?: boolean;

  defaultExpanded?: Record<Ref<App>, boolean>;
  interval: Interval;
}

export function Gantt2({
  usages,
  // usagesLoading,
  // interactionPeriods,
  // interactionPeriodsLoading,
  // systemEvents,
  // systemEventsLoading,
  // defaultExpanded,
  interval,
}: GanttProps) {
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);

  const appUsages = useMemo(
    () =>
      Object.entries(usages).map(([appId, sessions]) => {
        return {
          id: +appId as Ref<App>,
          usages: Object.values(sessions).flatMap((session) => session.usages),
        };
      }),
    [usages],
  );

  // TODO: find width another way
  const width = useWidth(chartRef);

  useEffect(() => {
    if (!chartRef.current) return;

    const chart = echarts.init(chartRef.current);
    chartInstanceRef.current = chart;

    const timeGap = minRenderTimeGap(interval, width, 100);
    const mergedAppUsages = appUsages.map((appUsage) => {
      return {
        id: appUsage.id,
        usages: mergedUsages(appUsage.usages, timeGap),
      };
    });

    function appSeriesData(
      mergedAppUsages: { id: Ref<App>; usages: Usage[] }[],
    ) {
      return mergedAppUsages.map((appUsage) => {
        return {
          data: appUsage.usages.map((usage) => [
            ticksToDateTime(usage.start).toMillis(),
            ticksToDateTime(usage.end).toMillis(),
          ]),
        } as echarts.CustomSeriesOption;
      });
    }

    // Create series data for each session
    const seriesData: echarts.CustomSeriesOption[] = appSeriesData(
      mergedAppUsages,
    ).map((series, index) => ({
      animation: false,
      type: "custom",
      progressive: 0,
      renderItem: (
        params: echarts.CustomSeriesRenderItemParams,
        api: echarts.CustomSeriesRenderItemAPI,
      ): echarts.CustomSeriesRenderItemReturn => {
        const start = api.value(0);
        const end = api.value(1);
        const y = api.coord([0, index]);
        const rowHeight = y[1] - api.coord([0, index + 1])[1];

        const x = api.coord([start, 0])[0];
        // minimum 1px width
        const width = Math.max(api.coord([end, 0])[0] - x, 1);

        const padding = 0.2;
        const rectShape = {
          x,
          y: y[1] + rowHeight * 0.5 + rowHeight * padding,
          width,
          height: rowHeight * (1 - padding * 2),
        };
        const shape = echarts.graphic.clipRectByRect(
          rectShape,
          params.coordSys as unknown as RectLike,
        );
        return (
          shape && {
            type: "rect" as const,
            shape,
            style: {
              fill: "#1890ff",
            },
          }
        );
      },
      ...series,
    }));

    const option: echarts.EChartsOption = {
      tooltip: {
        trigger: "item",
        formatter: (params) => {
          return ``;
        },
      },
      grid: {
        left: "3%",
        right: "4%",
        bottom: 0,
        containLabel: true,
      },
      xAxis: [
        {
          type: "time",
          position: "top",
          min: interval.start.toMillis(),
          max: interval.end.toMillis(),
        },
      ],
      yAxis: {
        type: "category",
        data: appUsages.map((appUsage) => appUsage.id),
        axisLabel: {
          interval: 0,
        },
      },
      dataZoom: [
        {
          type: "slider",
          xAxisIndex: [0, 0],
          filterMode: "weakFilter",
          top: 0,
        },
        {
          type: "inside",
          xAxisIndex: [0, 0],
          filterMode: "weakFilter",
          // zoomOnMouseWheel: "shift",
          // moveOnMouseWheel: false,
          // preventDefaultMouseMove: false,
        },
      ],
      series: seriesData,
    };

    chart.setOption(option);

    const handler = _.debounce((params: any) => {
      let diff = params.end - params.start;
      if (params.batch) {
        const v = params.batch[params.batch.length - 1];
        diff = v.end - v.start;
      }
      const timeGap = minRenderTimeGap(interval, width, diff);
      const mergedAppUsages = appUsages.map((appUsage) => {
        return {
          id: appUsage.id,
          usages: mergedUsages(appUsage.usages, timeGap),
        };
      });
      const seriesData: echarts.CustomSeriesOption[] =
        appSeriesData(mergedAppUsages);
      chart.setOption({
        series: seriesData,
      });
    }, 500);

    chart.on("datazoom", handler);

    const resizeObserver = new ResizeObserver(() => {
      requestAnimationFrame(() => chart.resize());
    });

    resizeObserver.observe(chartRef.current);

    return () => {
      chart.dispose();
      resizeObserver.disconnect();
    };
  }, [interval, appUsages]);

  return (
    <div className="w-full h-full">
      <div ref={chartRef} className="w-full h-full" />
    </div>
  );
}

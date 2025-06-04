import { useEffect, useMemo, useRef } from "react";
import * as echarts from "echarts";
import type { App, Ref, Usage } from "@/lib/entities";
import type { InteractionPeriod, SystemEvent } from "@/lib/entities";
import type { AppSessionUsages } from "@/lib/repo";
import { dateTimeToTicks, ticksToDateTime, type Interval } from "@/lib/time";
import { useWidth } from "@/hooks/use-width";
import _ from "lodash";
import { DateTime } from "luxon";

type RectLike = {
  x: number;
  y: number;
  width: number;
  height: number;
};

interface CombinedUsage {
  type: "combined";
  start: number;
  end: number;
  count: number;
}

type UsageBar = CombinedUsage | Usage;

const minRenderWidth = 1;
const maxRenderTimeGap = 1000000000;

function minRenderTimeGap(interval: Interval, width: number, dataZoom: number) {
  const timeGap = interval.end.diff(interval.start).toMillis();
  const zoom = dataZoom / 100;
  const minRenderTimeGapMillis = ((timeGap * zoom) / width) * minRenderWidth;
  const minRenderTimeGap = minRenderTimeGapMillis * 10_000;
  return Math.max(minRenderTimeGap, 1);
}

function mergedUsages(
  usages: Usage[],
  minRenderTimeGap: number,
  start: DateTime,
  end: DateTime,
): UsageBar[] {
  const startTicks = dateTimeToTicks(start);
  const endTicks = dateTimeToTicks(end);
  usages = usages.filter(
    (usage) => usage.end >= startTicks && usage.start <= endTicks,
  );
  usages.sort((a, b) => a.start - b.start);
  if (minRenderTimeGap < maxRenderTimeGap) {
    return usages;
  }
  const mergedUsages: UsageBar[] = [{ ...usages[0] }];
  for (const usage of usages) {
    let lastUsage = mergedUsages[mergedUsages.length - 1];
    if (lastUsage.end + minRenderTimeGap > usage.start) {
      const lastUsageBar = lastUsage as CombinedUsage;
      if (lastUsageBar.type !== "combined") {
        lastUsageBar.type = "combined";
        lastUsageBar.count = 1;
      }
      lastUsage.end = usage.end;
      lastUsageBar.count += 1;
    } else {
      mergedUsages.push({ ...usage });
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
  appBarHeight?: number;
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
  appBarHeight = 44,
}: GanttProps) {
  const chartRef = useRef<HTMLDivElement>(null);
  const topRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);
  const topInstanceRef = useRef<echarts.ECharts | null>(null);

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
    if (!topRef.current) return;

    const chart = echarts.init(chartRef.current);
    chartInstanceRef.current = chart;

    const top = echarts.init(topRef.current);
    topInstanceRef.current = top;

    const timeGap = minRenderTimeGap(interval, width, 100);
    const mergedAppUsages = appUsages.map((appUsage) => {
      return {
        id: appUsage.id,
        usages: mergedUsages(
          appUsage.usages,
          timeGap,
          interval.start,
          interval.end,
        ),
      };
    });

    function appSeriesData(
      mergedAppUsages: { id: Ref<App>; usages: UsageBar[] }[],
    ) {
      return mergedAppUsages.map((appUsage) => {
        return {
          data: appUsage.usages.map((usage) => [
            ticksToDateTime(usage.start).toMillis(),
            ticksToDateTime(usage.end).toMillis(),
            (usage as CombinedUsage).count,
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

    const common = {
      animation: false,
      grid: {
        left: 5,
        right: 5,
        bottom: 0,
        containLabel: true,
      },
      xAxis: [
        {
          id: "timeAxis",
          type: "time",
          position: "top",
          min: interval.start.toMillis(),
          max: interval.end.toMillis(),
        },
      ],
      yAxis: {
        show: false,
        type: "category",
        data: appUsages.map((appUsage) => appUsage.id),
        axisLabel: {
          interval: 0,
          show: false,
        },
      },
      dataZoom: [
        {
          id: "dataZoomSlider",
          type: "slider",
          xAxisIndex: [0],
          filterMode: "none",
          top: 0,
        },
        {
          id: "dataZoomInside",
          type: "inside",
          xAxisIndex: [0],
          filterMode: "none",
          // zoomOnMouseWheel: "shift",
          // moveOnMouseWheel: false,
          // preventDefaultMouseMove: false,
        },
      ],
    } satisfies echarts.EChartsOption;

    const optionTop: echarts.EChartsOption = {
      ...common,
      xAxis: [
        {
          ...common.xAxis[0],
          show: true,
          minorTick: {
            show: true,
          },
          axisTick: {
            show: true,
          },
          alignTicks: true,
        },
      ],
    };

    const option: echarts.EChartsOption = {
      ...common,
      tooltip: {
        trigger: "item",
        formatter: (params) => {
          if (!params.data) {
            return "";
          }
          const [startMillis, endMillis, count] = params.data;
          const start = DateTime.fromMillis(startMillis);
          const end = DateTime.fromMillis(endMillis);

          const title = count ? `Multiple Usages: ${count}` : "Single Usage";

          return `${title}<br/>Start: ${start.toFormat("yyyy-MM-dd HH:mm:ss.SSS")}<br/>End: ${end.toFormat("yyyy-MM-dd HH:mm:ss.SSS")}`;
        },
      },
      grid: {
        ...common.grid,
        top: 0,
      },
      xAxis: [
        {
          ...common.xAxis[0],
          show: false,
          axisLabel: {
            show: false,
          },
        },
      ],
      dataZoom: [
        {
          ...common.dataZoom[0],
          show: false,
        },
        {
          ...common.dataZoom[1],
          show: false,
        },
      ],
      series: seriesData,
    };

    chart.setOption(option);
    top.setOption(optionTop);
    echarts.connect([chart, top]);

    const handler = _.debounce((params: any) => {
      let diff = params.end - params.start;
      let start = params.startValue
        ? DateTime.fromMillis(params.startValue)
        : null;
      let end = params.endValue ? DateTime.fromMillis(params.endValue) : null;
      if (params.batch) {
        const v = params.batch[params.batch.length - 1];
        diff = v.end - v.start;
        start = interval.start.plus({
          milliseconds:
            interval.end.diff(interval.start).toMillis() * (v.start / 100),
        });
        end = interval.start.plus({
          milliseconds:
            interval.end.diff(interval.start).toMillis() * (v.end / 100),
        });
      }
      const timeGap = minRenderTimeGap(interval, width, diff);
      const mergedAppUsages = appUsages.map((appUsage) => {
        return {
          id: appUsage.id,
          usages: mergedUsages(appUsage.usages, timeGap, start!, end!),
        };
      });
      const seriesData: echarts.CustomSeriesOption[] =
        appSeriesData(mergedAppUsages);
      chart.setOption({
        series: seriesData,
      });
    }, 50);

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
    <div className="w-full h-full sticky">
      <div
        ref={topRef}
        className="sticky z-10 w-full border-border border-b bg-card top-0"
        style={{ height: 90 }}
      />
      <div
        ref={chartRef}
        className="w-full"
        style={{ height: appBarHeight * appUsages.length }}
      />
    </div>
  );
}

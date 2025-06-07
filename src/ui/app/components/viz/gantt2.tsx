import { useEffect, useMemo, useRef } from "react";
import * as echarts from "echarts";
import type { App, Ref, Usage } from "@/lib/entities";
import type { InteractionPeriod, SystemEvent } from "@/lib/entities";
import type { AppSessionUsages } from "@/lib/repo";
import {
  ticksToDateTime,
  unixMillisToTicks,
  type Interval,
} from "@/lib/time";
import { useWidth } from "@/hooks/use-width";
import _ from "lodash";
import { DateTime } from "luxon";
import { useApps } from "@/hooks/use-refresh";
import { ChevronDown, ChevronRight } from "lucide-react";
import AppIcon from "@/components/app/app-icon";
import { Text } from "@/components/ui/text";
import { getVarColorAsHex } from "@/lib/color-utils";

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
  start: number,
  end: number,
): UsageBar[] {
  const startTicks = unixMillisToTicks(start);
  const endTicks = unixMillisToTicks(end);
  usages = usages.filter(
    (usage) => usage.end >= startTicks && usage.start <= endTicks,
  );
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
  infoGap?: number;
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
  infoGap = 300,
  appBarHeight = 52,
}: GanttProps) {
  const chartRef = useRef<HTMLDivElement>(null);
  const topRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);
  const topInstanceRef = useRef<echarts.ECharts | null>(null);

  const appIds = useMemo(() => {
    return Object.keys(usages).map((appId) => +appId as Ref<App>);
  }, [usages]);
  const apps = useApps(appIds);

  const appUsages = useMemo(
    () =>
      apps.map((app) => ({
        id: app.id,
        usages: Object.values(usages[app.id])
          .flatMap((session) => session.usages)
          .sort((a, b) => a.start - b.start),
      })),
    [usages, apps],
  );

  // TODO: find width another way
  const width = useWidth(chartRef);

  useEffect(() => {
    if (!chartRef.current) return;
    if (!topRef.current) return;

    const intervalDurationMillis = interval.end.diff(interval.start).toMillis();
    const intervalStartMillis = interval.start.toMillis();

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
          interval.start.toMillis(),
          interval.end.toMillis(),
        ),
      };
    });

    function appSeriesData(
      mergedAppUsages: { id: Ref<App>; usages: UsageBar[] }[],
    ) {
      return mergedAppUsages.reverse().map((appUsage) => {
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
          y: y[1] - rowHeight * 0.5 + rowHeight * padding,
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
        left: infoGap,
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
        axisLine: {
          show: false,
        },
        axisTick: {
          show: false,
        },
      },
      dataZoom: [
        {
          id: "dataZoomSlider",
          type: "slider",
          xAxisIndex: [0],
          filterMode: "none",
          top: 5,
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
            inside: true,
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
          axisLabel: {
            show: false,
          },
          splitLine: {
            show: true,
            showMaxLine: false,
            showMinLine: false,
            lineStyle: {
              opacity: 0.4,
              color: getVarColorAsHex("foreground"),
              type: [1, 5],
            },
          },
        },
      ],
      yAxis: {
        ...common.yAxis,
        show: true,
        splitLine: {
          show: true,
          lineStyle: {
            opacity: 0.5,
            color: getVarColorAsHex("border"),
            type: "solid",
          },
        },
      },
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
      if (params.batch) {
        params = params.batch[params.batch.length - 1];
      }

      // percentage (100)
      const diff = params.end - params.start;
      // in millis
      const start =
        params.startValue ??
        intervalStartMillis + intervalDurationMillis * (params.start / 100);
      const end =
        params.endValue ??
        intervalStartMillis + intervalDurationMillis * (params.end / 100);

      const timeGap = minRenderTimeGap(interval, width, diff);

      const mergedAppUsages = appUsages.map((appUsage) => {
        return {
          id: appUsage.id,
          usages: mergedUsages(appUsage.usages, timeGap, start, end),
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
      requestAnimationFrame(() => top.resize());
      requestAnimationFrame(() => chart.resize());
    });

    // if chartRef changes, so does topRef
    resizeObserver.observe(chartRef.current);

    return () => {
      top.dispose();
      chart.dispose();
      resizeObserver.disconnect();
    };
  }, [interval, appUsages]);

  return (
    <div className="w-full h-full sticky">
      <div
        ref={topRef}
        className="sticky z-10 w-full border-border border-b bg-card top-0 shadow-md dark:shadow-xl"
        style={{ height: 90 }}
      />
      <div className="relative">
        <div
          ref={chartRef}
          className="w-full"
          style={{ height: appBarHeight * appUsages.length }}
        />
        <div
          className="absolute top-0 left-0 bottom-0"
          style={{ width: infoGap }}
        >
          {apps.map((app) => (
            <div key={app.id} className="relative">
              <div
                className="flex items-center p-4 bg-muted/80 hover:bg-muted/60 border-r"
                style={{ height: appBarHeight }}
                // onClick={() => toggleApp(app.id)}
              >
                {/* {expanded[app.id] ? ( */}
                {true ? (
                  <ChevronDown size={20} className="flex-shrink-0" />
                ) : (
                  <ChevronRight size={20} className="flex-shrink-0" />
                )}
                <AppIcon buffer={app.icon} className="ml-2 w-6 h-6 shrink-0" />
                <Text className="font-semibold ml-4">{app.name}</Text>
              </div>
              <div className="h-px bg-border absolute bottom-[-0.5px] left-0 right-0" />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

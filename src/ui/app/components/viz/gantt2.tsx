import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as echarts from "echarts";
import type { App, Ref, Session, Usage } from "@/lib/entities";
import type { InteractionPeriod, SystemEvent } from "@/lib/entities";
import type { AppSessionUsages } from "@/lib/repo";
import { ticksToUnixMillis, type Interval } from "@/lib/time";
import { useWidth } from "@/hooks/use-width";
import _ from "lodash";
import { DateTime } from "luxon";
import { useApps } from "@/hooks/use-refresh";
import { ChevronDown, ChevronRight } from "lucide-react";
import AppIcon from "@/components/app/app-icon";
import { Text } from "@/components/ui/text";
import { getVarColorAsHex } from "@/lib/color-utils";
import { useDebounce } from "use-debounce";
import { useAppState } from "@/lib/state";
import { DateTimeText } from "../time/time-text";

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

interface SessionSeriesKey {
  type: "session";
  id: Ref<Session>;
  appId: Ref<App>;
  y: number;
}

interface AppSeriesKey {
  type: "app";
  id: Ref<App>;
  y: number;
}

type SeriesKey = SessionSeriesKey | AppSeriesKey;
type UsageBar = CombinedUsage | Usage;

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
  sessionBarHeight?: number;
  infoGap?: number;
}

export function Gantt2({
  usages: usagesPerAppSession,
  // usagesLoading,
  // interactionPeriods,
  // interactionPeriodsLoading,
  // systemEvents,
  // systemEventsLoading,
  defaultExpanded,
  interval,
  infoGap = 300,
  appBarHeight = 52,
  sessionBarHeight = 84,
}: GanttProps) {
  const chartRef = useRef<HTMLDivElement>(null);
  const topRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);
  const topInstanceRef = useRef<echarts.ECharts | null>(null);

  const [expanded, setExpanded] = useState<Record<Ref<App>, boolean>>(
    defaultExpanded ?? {},
  );
  const [chartInit, setChartInit] = useState(false);
  const [dataZoom, setDataZoom] = useDebounce(100, 200);

  const appIds = useMemo(() => {
    return Object.keys(usagesPerAppSession).map((appId) => +appId as Ref<App>);
  }, [usagesPerAppSession]);
  const apps = useApps(appIds);
  const appMap = useAppState((state) => state.apps);

  const usagesPerApp: Record<Ref<App>, Usage[]> = useMemo(
    () =>
      _(apps)
        .map(
          (app) =>
            [
              app.id,
              Object.values(usagesPerAppSession[app.id])
                .flatMap((session) => session.usages)
                .sort((a, b) => a.start - b.start),
            ] as const,
        )
        .fromPairs()
        .value(),
    [usagesPerAppSession, apps],
  );

  const toggleApp = useCallback(
    (appId: Ref<App>) => {
      setExpanded((prev) => ({
        ...prev,
        [appId]: !prev[appId],
      }));
    },
    [setExpanded],
  );

  // TODO: find width another way
  const width = useWidth(chartRef);

  const [seriesKeys, seriesHeight] = useMemo(
    () =>
      getSeriesKeys(
        expanded,
        apps,
        usagesPerAppSession,
        appBarHeight,
        sessionBarHeight,
      ),
    [expanded, apps, usagesPerAppSession, appBarHeight, sessionBarHeight],
  );

  const seriesKeyToSeries = useCallback(
    (
      key: SeriesKey,
      timeGap: number,
      color: string,
    ): echarts.CustomSeriesOption => {
      const id = key.type + key.id;
      if (key.type === "app") {
        const usages = mergedUsages(usagesPerApp[key.id], timeGap);
        return {
          ...appBar(appBarHeight, sessionBarHeight, color),
          id,
          encode: {
            x: [1, 2],
            y: 0,
          },
          data: usages.map((usage) => [
            key.y,
            ticksToUnixMillis(usage.start),
            ticksToUnixMillis(usage.end),
            (usage as CombinedUsage).count,
            key.id,
            key.type,
          ]),
        } satisfies echarts.CustomSeriesOption;
      } else {
        const usages = mergedUsages(
          usagesPerAppSession[key.appId][key.id].usages,
          timeGap,
        );
        return {
          ...appBar(appBarHeight, sessionBarHeight, color),
          id,
          encode: {
            x: [1, 2],
            y: 0,
          },
          data: usages.map((usage) => [
            key.y,
            ticksToUnixMillis(usage.start),
            ticksToUnixMillis(usage.end),
            (usage as CombinedUsage).count,
            key.id,
            key.type,
          ]),
        } satisfies echarts.CustomSeriesOption;
      }
    },
    [usagesPerApp, usagesPerAppSession, appBarHeight, sessionBarHeight],
  );

  useEffect(() => {
    if (!chartInit) return;

    const timeGap = minRenderTimeGap(interval, width, dataZoom);
    const color = getVarColorAsHex("primary");
    const series = seriesKeys.map((key) =>
      seriesKeyToSeries(key, timeGap, color),
    );

    const commonOptions = {
      grid: {
        left: infoGap,
      },
      xAxis: {
        min: interval.start.toMillis(),
        max: interval.end.toMillis(),
      },
      yAxis: {
        min: 0,
        max: seriesHeight,
        axisTick: {
          customValues: seriesKeys.map((key) => key.y).filter((y) => y !== 0), // skip first one
        },
      },
    } satisfies echarts.EChartsOption;

    topInstanceRef.current?.setOption({
      ...commonOptions,
    });

    chartInstanceRef.current?.setOption(
      {
        ...commonOptions,
        series,
      },
      {
        replaceMerge: ["series"],
      },
    );

    // Force resize now instead of waiting for the next frame
    topInstanceRef.current?.resize();
    chartInstanceRef.current?.resize();
  }, [
    seriesKeys,
    seriesKeyToSeries,
    width,
    interval,
    infoGap,
    seriesHeight,
    chartInit,
    dataZoom,
  ]);

  useEffect(() => {
    if (!chartRef.current) return;
    if (!topRef.current) return;

    const chart = echarts.init(chartRef.current);
    chartInstanceRef.current = chart;

    const top = echarts.init(topRef.current);
    topInstanceRef.current = top;

    const common = {
      animation: false,
      grid: {
        right: 5,
        bottom: 0,
        containLabel: true,
      },
      xAxis: {
        id: "timeAxis",
        type: "time",
        position: "top",
      },
      yAxis: {
        show: false,
        type: "value",
        inverse: true,
        splitLine: { show: false },
        axisLabel: {
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
          filterMode: "weakFilter",
          top: 5,
        },
        {
          id: "dataZoomInside",
          type: "inside",
          xAxisIndex: [0],
          filterMode: "weakFilter",
          // zoomOnMouseWheel: "shift",
          // moveOnMouseWheel: false,
          // preventDefaultMouseMove: false,
        },
      ],
    } satisfies echarts.EChartsOption;

    const optionTop: echarts.EChartsOption = {
      ...common,
      xAxis: {
        ...common.xAxis,
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
    };

    const option: echarts.EChartsOption = {
      ...common,
      tooltip: {
        trigger: "item",
        formatter: (params) => {
          if (!params.data) {
            return "";
          }
          const [id, startMillis, endMillis, count] = params.data;
          const start = DateTime.fromMillis(startMillis);
          const end = DateTime.fromMillis(endMillis);

          const title = count ? `Multiple Usages: ${count}` : "Single Usage";

          return `${title} - ${id}<br/>Start: ${start.toFormat("yyyy-MM-dd HH:mm:ss.SSS")}<br/>End: ${end.toFormat("yyyy-MM-dd HH:mm:ss.SSS")}`;
        },
      },
      grid: {
        ...common.grid,
        top: 0,
      },
      xAxis: {
        ...common.xAxis,
        axisLine: {
          show: false,
        },
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
    };

    chart.setOption(option);
    top.setOption(optionTop);
    echarts.connect([chart, top]);

    const handler = (params: any) => {
      if (params.batch) {
        params = params.batch[params.batch.length - 1];
      }

      // percentage (100)
      const diff = params.end - params.start;
      setDataZoom(diff);
    };

    chart.on("datazoom", handler);

    const resizeObserver = new ResizeObserver(() => {
      requestAnimationFrame(() => top.resize());
      requestAnimationFrame(() => chart.resize());
    });

    // if chartRef changes, so does topRef
    resizeObserver.observe(chartRef.current);

    setChartInit(true);

    return () => {
      resizeObserver.disconnect();
      top.dispose();
      chart.dispose();
    };
  }, [setDataZoom]);

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
          style={{
            height: seriesHeight,
          }}
        />
        <div
          className="absolute top-0 left-0 bottom-0"
          style={{ width: infoGap }}
        >
          {seriesKeys.map((key) =>
            key.type === "app" ? (
              <div key={key.type + key.id} className="relative">
                <div
                  className="flex items-center p-4 bg-muted/80 hover:bg-muted/60 border-r"
                  style={{ height: appBarHeight }}
                  onClick={() => toggleApp(key.id)}
                >
                  {expanded[key.id] ? (
                    <ChevronDown size={20} className="flex-shrink-0" />
                  ) : (
                    <ChevronRight size={20} className="flex-shrink-0" />
                  )}
                  <AppIcon
                    buffer={appMap[key.id]?.icon}
                    className="ml-2 w-6 h-6 shrink-0"
                  />
                  <Text className="font-semibold ml-4">
                    {appMap[key.id]?.name ?? ""}
                  </Text>
                </div>
                <div className="h-px bg-border absolute bottom-[-0.5px] left-0 right-0" />
              </div>
            ) : (
              <div className="relative" key={key.type + key.id}>
                <div
                  className="p-2 border-t border-r flex flex-col justify-center"
                  style={{ height: sessionBarHeight }}
                >
                  <Text className="text-sm">
                    {usagesPerAppSession[key.appId][key.id].title}
                  </Text>
                  {usagesPerAppSession[key.appId][key.id].url && (
                    <Text className="text-xs font-mono text-muted-foreground">
                      {usagesPerAppSession[key.appId][key.id].url ?? ""}
                    </Text>
                  )}
                  <div className="text-xs text-muted-foreground inline-flex gap-1 items-center">
                    <DateTimeText
                      ticks={usagesPerAppSession[key.appId][key.id].start}
                    />
                    <span className="text-muted-foreground">-</span>
                    <DateTimeText
                      ticks={usagesPerAppSession[key.appId][key.id].end}
                      className="text-xs"
                    />
                  </div>
                </div>
                <div className="h-px bg-border absolute bottom-[-0.5px] left-0 right-0" />
              </div>
            ),
          )}
        </div>
      </div>
    </div>
  );
}

export const minRenderWidth = 1;
export const maxRenderTimeGap = 600_000_000; // 1 minute

export function minRenderTimeGap(
  interval: Interval,
  width: number,
  dataZoom: number,
) {
  const timeGap = interval.end.diff(interval.start).toMillis();
  const zoom = dataZoom / 100;
  const minRenderTimeGapMillis = ((timeGap * zoom) / width) * minRenderWidth;
  const minRenderTimeGap = minRenderTimeGapMillis * 10_000;
  return Math.max(minRenderTimeGap, 1);
}

export function mergedUsages(
  usages: Usage[],
  minRenderTimeGap: number,
): UsageBar[] {
  if (minRenderTimeGap < maxRenderTimeGap) {
    return usages;
  }
  const mergedUsages: UsageBar[] = [{ ...usages[0] }];
  for (const usage of usages) {
    const lastUsage = mergedUsages[mergedUsages.length - 1];
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

function appBar(
  appBarHeight: number,
  sessionBarHeight: number,
  color: string,
): echarts.CustomSeriesOption {
  return {
    animation: false,
    type: "custom",
    progressive: 0,
    renderItem: (
      params: echarts.CustomSeriesRenderItemParams,
      api: echarts.CustomSeriesRenderItemAPI,
    ): echarts.CustomSeriesRenderItemReturn => {
      const index = api.value(0);
      const start = api.value(1);
      const end = api.value(2);
      const type = api.value(5);

      const rowHeight = type === "app" ? appBarHeight : sessionBarHeight;
      const height = 24;

      const [x, y] = api.coord([start, index]);
      // minimum 1px width
      const width = Math.max(api.coord([end, index])[0] - x, 1);

      const rectShape = {
        x,
        y: y + (rowHeight - height) / 2,
        width,
        height,
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
            fill: color,
          },
        }
      );
    },
  };
}

function getSeriesKeys(
  expanded: Record<Ref<App>, boolean>,
  apps: App[],
  usagesPerAppSession: AppSessionUsages,
  appBarHeight: number,
  sessionBarHeight: number,
): [SeriesKey[], number] {
  const seriesKeys: SeriesKey[] = [];
  let y = 0;
  for (const app of apps) {
    seriesKeys.push({ type: "app", id: app.id, y });
    y += appBarHeight;

    if (expanded[app.id]) {
      for (const sessionId of Object.keys(usagesPerAppSession[app.id])) {
        const session = +sessionId as Ref<Session>;
        seriesKeys.push({ type: "session", id: session, appId: app.id, y });
        y += sessionBarHeight;
      }
    }
  }
  return [seriesKeys, y];
}

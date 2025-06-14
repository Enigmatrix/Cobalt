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
}

interface AppSeriesKey {
  type: "app";
  id: Ref<App>;
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
  sessionBarHeight = 52,
}: GanttProps) {
  const chartRef = useRef<HTMLDivElement>(null);
  const topRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);
  const topInstanceRef = useRef<echarts.ECharts | null>(null);

  const [expanded, setExpanded] = useState<Record<Ref<App>, boolean>>(
    defaultExpanded ?? {},
  );

  const appIds = useMemo(() => {
    return Object.keys(usagesPerAppSession).map((appId) => +appId as Ref<App>);
  }, [usagesPerAppSession]);
  const apps = useApps(appIds);

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

  const seriesKeys = useMemo(
    () => getSeriesKeys(expanded, apps, usagesPerAppSession),
    [expanded, apps, usagesPerAppSession],
  );

  useEffect(() => {
    if (!chartRef.current) return;
    if (!topRef.current) return;

    const chart = echarts.init(chartRef.current);
    chartInstanceRef.current = chart;

    const top = echarts.init(topRef.current);
    topInstanceRef.current = top;

    const timeGap = minRenderTimeGap(interval, width, 100);

    function seriesKeyToSeries(
      key: SeriesKey,
      timeGap: number,
    ): echarts.CustomSeriesOption {
      const id = key.type + key.id;
      if (key.type === "app") {
        const usages = mergedUsages(usagesPerApp[key.id], timeGap);
        return {
          ...appBar(),
          id,
          encode: {
            x: [1, 2],
            y: 0,
          },
          data: usages.map((usage) => [
            id,
            ticksToUnixMillis(usage.start),
            ticksToUnixMillis(usage.end),
            (usage as CombinedUsage).count,
          ]),
        } satisfies echarts.CustomSeriesOption;
      } else {
        // return sessionBar(key.id);
        throw new Error("Not implemented");
      }
    }

    function seriesKeyToSeriesData(
      key: SeriesKey,
      timeGap: number,
    ): echarts.CustomSeriesOption {
      const id = key.type + key.id;
      if (key.type === "app") {
        const usages = mergedUsages(usagesPerApp[key.id], timeGap);

        return {
          data: usages.map((usage) => [
            id,
            ticksToUnixMillis(usage.start),
            ticksToUnixMillis(usage.end),
            (usage as CombinedUsage).count,
          ]),
        } satisfies echarts.CustomSeriesOption;
      } else {
        // return sessionBar(key.id);
        throw new Error("Not implemented");
      }
    }

    const series: echarts.CustomSeriesOption[] = seriesKeys.map((key) =>
      seriesKeyToSeries(key, timeGap),
    );

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
        // setting both data and inverse makes it correct.
        data: seriesKeys.map((key) => key.type + key.id),
        inverse: true,

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
      series,
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
      const timeGap = minRenderTimeGap(interval, width, diff);

      chart.setOption({
        series: seriesKeys.map((key) => seriesKeyToSeriesData(key, timeGap)),
      });
    }, 200);

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
  }, [interval, usagesPerApp]);

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
            height: getSeriesSize(seriesKeys, appBarHeight, sessionBarHeight),
          }}
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
                onClick={() => toggleApp(app.id)}
              >
                {expanded[app.id] ? (
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

function appBar(): echarts.CustomSeriesOption {
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

      const rowHeight = (api.size!([0, 1]) as number[])[1];

      const [x, y] = api.coord([start, index]);
      // minimum 1px width
      const width = Math.max(api.coord([end, index])[0] - x, 1);

      const padding = 0.2;
      const rectShape = {
        x,
        y: y - rowHeight * 0.5 + rowHeight * padding,
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
  };
}

function getSeriesKeys(
  expanded: Record<Ref<App>, boolean>,
  apps: App[],
  usagesPerAppSession: AppSessionUsages,
): SeriesKey[] {
  const seriesKeys: SeriesKey[] = [];
  for (const app of apps) {
    if (expanded[app.id]) {
      for (const sessionId of Object.keys(usagesPerAppSession[app.id])) {
        const session = +sessionId as Ref<Session>;
        seriesKeys.push({ type: "session", id: session, appId: app.id });
      }
    }
    seriesKeys.push({ type: "app", id: app.id });
  }
  return seriesKeys;
}

function getSeriesSize(
  seriesKeys: SeriesKey[],
  appBarHeight: number,
  sessionBarHeight: number,
) {
  const appSeriesCount = seriesKeys.filter((key) => key.type === "app").length;
  const sessionSeriesCount = seriesKeys.filter(
    (key) => key.type === "session",
  ).length;
  return appSeriesCount * appBarHeight + sessionSeriesCount * sessionBarHeight;
}

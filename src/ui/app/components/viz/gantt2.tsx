import AppIcon from "@/components/app/app-icon";
import {
  DISTRACTIVE_STREAK_BACKGROUND,
  FOCUS_STREAK_BACKGROUND,
} from "@/components/tag/score";
import { DurationText } from "@/components/time/duration-text";
import { DateTimeText } from "@/components/time/time-text";
import { Text } from "@/components/ui/text";
import { VizTooltip } from "@/components/viz/viz-tooltip";
import { useApps } from "@/hooks/use-refresh";
import { useStreakDurations } from "@/hooks/use-repo";
import { useWidth } from "@/hooks/use-width";
import { getVarColorAsHex } from "@/lib/color-utils";
import {
  systemEventToString,
  type App,
  type Ref,
  type Session,
  type SystemEventEnum,
} from "@/lib/entities";
import type { InteractionPeriod, Streak, SystemEvent } from "@/lib/entities";
import type { AppSessionUsages } from "@/lib/repo";
import { useAppState } from "@/lib/state";
import {
  TICKS_PER_MILLISECOND,
  ticksToUnixMillis,
  unixMillisToTicks,
  type Interval,
} from "@/lib/time";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import * as echarts from "echarts";
import _ from "lodash";
import {
  ChevronDown,
  ChevronRight,
  KeyboardIcon,
  LaptopIcon,
  Loader2Icon,
  MouseIcon,
} from "lucide-react";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import { VariableSizeList as List } from "react-window";
import { useDebounce } from "use-debounce";

const DATAZOOM_SLIDER_X = "dataZoomSliderX";
const DATAZOOM_INSIDE_X = "dataZoomInsideX";
const DATAZOOM_INSIDE_Y = "dataZoomInsideY";

interface RectLike {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface DataZoomEvent {
  batch: DataZoomEvent[];
  start: number;
  end: number;
  startValue: number;
  endValue: number;
  id?: string;
  dataZoomId?: string;
}

interface CombinedUsage {
  type: "combined";
  start: number;
  end: number;
  count: number;
}

interface AppSessionUsage {
  start: number;
  end: number;
  sessionId: Ref<Session>;
  appId: Ref<App>;
}

type UsageBar = CombinedUsage | AppSessionUsage;

interface CombinedInteractionPeriod {
  type: "combined";
  start: number;
  end: number;
  mouseClicks: number;
  keyStrokes: number;
  count: number;
}

type InteractionBar = CombinedInteractionPeriod | InteractionPeriod;

interface SessionSeriesKey {
  type: "session";
  id: Ref<Session>;
  appId: Ref<App>;
}

interface AppSeriesKey {
  type: "app";
  id: Ref<App>;
}

interface InteractionBarSeriesKey {
  type: "interactionBar";
  id: 0;
}

type SeriesKey = (SessionSeriesKey | AppSeriesKey | InteractionBarSeriesKey) & {
  y: number;
  top: number;
  bottom: number;
};

interface GanttProps {
  usages: AppSessionUsages;
  usagesLoading?: boolean;
  usagesValidating?: boolean;

  streaks?: Streak[];
  streaksLoading?: boolean;
  streaksValidating?: boolean;

  interactionPeriods?: InteractionPeriod[];
  interactionPeriodsLoading?: boolean;
  interactionPeriodsValidating?: boolean;

  systemEvents?: SystemEvent[];
  systemEventsLoading?: boolean;
  systemEventsValidating?: boolean;

  summary?: React.ReactNode;
  defaultExpanded?: Record<Ref<App>, boolean>;
  interval: Interval;
  interactionInfoBarHeight?: number;
  appInfoBarHeight?: number;
  sessionInfoBarHeight?: number;
  sessionInfoUrlBarHeight?: number;
  infoGap?: number;
  innerHeight?: number;
  scrollbarWidth?: number;
}

export function Gantt({
  usages: usagesPerAppSession,
  usagesLoading,
  // we don't actually use this - the indicator is in the Session cards instead
  // usagesValidating,
  streaks,
  streaksLoading,
  streaksValidating,
  interactionPeriods,
  interactionPeriodsLoading,
  interactionPeriodsValidating,
  systemEvents,
  systemEventsLoading,
  systemEventsValidating,
  summary,
  defaultExpanded,
  interval,
  infoGap = 300,
  interactionInfoBarHeight = 52,
  appInfoBarHeight = 52,
  sessionInfoBarHeight = 72,
  sessionInfoUrlBarHeight = 84,
  innerHeight = 750,
  scrollbarWidth = 15,
}: GanttProps) {
  const chartRef = useRef<HTMLDivElement>(null);
  const topRef = useRef<HTMLDivElement>(null);
  const infoBarsRef = useRef<List>(null);
  const scrollbarRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);
  const topInstanceRef = useRef<echarts.ECharts | null>(null);

  const [hoverSystemEvent, setHoverSystemEvent] = useState<SystemEvent | null>(
    null,
  );
  const [hoverData, setHoverData] = useState<UsageBar | null>(null);
  const [hoverInteraction, setHoverInteraction] =
    useState<InteractionBar | null>(null);
  const [expanded, setExpanded] = useState<Record<Ref<App>, boolean>>(
    defaultExpanded ?? {},
  );
  const [chartInit, setChartInit] = useState(false);
  const [dataZoom, setDataZoom] = useDebounce(100, 200);

  // Sort apps by total usage duration - this might be slow?
  const appIds = useMemo(() => {
    return _(usagesPerAppSession)
      .mapValues(
        (sessions, appId) =>
          [
            +appId as Ref<App>,
            _(sessions)
              .flatMap((session) =>
                session.usages.map((usage) => usage.end - usage.start),
              )
              .sum(),
          ] as const,
      )
      .sortBy(([, duration]) => -duration)
      .map(([appId]) => appId)
      .value();
  }, [usagesPerAppSession]);
  const apps = useApps(appIds);
  const appMap = useAppState((state) => state.apps);

  const usagesPerApp: Record<Ref<App>, AppSessionUsage[]> = useMemo(
    () =>
      _(apps)
        .map(
          (app) =>
            [
              app.id,
              Object.values(usagesPerAppSession[app.id])
                .flatMap((session) =>
                  session.usages.map((usage) => ({
                    start: usage.start,
                    end: usage.end,
                    sessionId: session.id,
                    appId: app.id,
                  })),
                )
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
        !!(interactionPeriods ?? systemEvents),
        interactionInfoBarHeight,
        appInfoBarHeight,
        sessionInfoBarHeight,
        sessionInfoUrlBarHeight,
        24,
      ),
    [
      expanded,
      apps,
      usagesPerAppSession,
      interactionPeriods,
      systemEvents,
      interactionInfoBarHeight,
      appInfoBarHeight,
      sessionInfoBarHeight,
      sessionInfoUrlBarHeight,
    ],
  );

  const seriesKeyToSeries = useCallback(
    (
      key: SeriesKey,
      timeGap: number,
      color: string,
    ): echarts.CustomSeriesOption => {
      const backgroundColor = getVarColorAsHex("muted");

      const id = key.type + key.id;
      if (key.type === "app") {
        const usages = mergedUsages(usagesPerApp[key.id], timeGap);
        return {
          ...appBar(
            color,
            key.y,
            key.y + appInfoBarHeight,
            interval,
            backgroundColor,
          ),
          id,
          encode: {
            x: [2, 3],
            y: [0, 1],
          },
          data: usages.map((usage) => [
            key.top,
            key.bottom,
            ticksToUnixMillis(usage.start),
            ticksToUnixMillis(usage.end),
            key.type,
            key.id,
            false,
            (usage as CombinedUsage).count,
            undefined,
            (usage as AppSessionUsage).sessionId,
            (usage as AppSessionUsage).appId,
          ]),
        } satisfies echarts.CustomSeriesOption;
      } else if (key.type === "session") {
        const usages = mergedUsages(
          usagesPerAppSession[key.appId][key.id].usages.map((usage) => ({
            start: usage.start,
            end: usage.end,
            sessionId: key.id,
            appId: key.appId,
          })),
          timeGap,
        );
        return {
          ...seriesBar(color),
          id,
          encode: {
            x: [2, 3],
            y: [0, 1],
          },
          data: usages.map((usage) => [
            key.top,
            key.bottom,
            ticksToUnixMillis(usage.start),
            ticksToUnixMillis(usage.end),
            key.type,
            key.id,
            usagesPerAppSession[key.appId][key.id].url,
            (usage as CombinedUsage).count,
            undefined,
            (usage as AppSessionUsage).sessionId,
            (usage as AppSessionUsage).appId,
          ]),
        } satisfies echarts.CustomSeriesOption;
      } else if (key.type === "interactionBar") {
        const mergedInteractionPeriodsData = mergedInteractionPeriods(
          interactionPeriods ?? [],
          timeGap,
        );
        return {
          ...interactionBar(color, "#ff6900"),
          id,
          encode: {
            x: [2, 3],
            y: [0, 1],
          },
          data: mergedInteractionPeriodsData
            .map((interactionPeriod) => [
              key.top,
              key.bottom,
              ticksToUnixMillis(interactionPeriod.start),
              ticksToUnixMillis(interactionPeriod.end),
              key.type,
              key.id,
              false,
              (interactionPeriod as CombinedInteractionPeriod).count,
              (interactionPeriod as InteractionPeriod).id,
              interactionPeriod.mouseClicks,
              interactionPeriod.keyStrokes,
            ])
            .concat(
              systemEvents?.map((systemEvent) => [
                key.top,
                key.bottom,
                ticksToUnixMillis(systemEvent.timestamp),
                ticksToUnixMillis(systemEvent.timestamp),
                key.type,
                key.id,
                true, // system event
                1,
                systemEvent.id,
                systemEvent.event,
              ]) ?? [],
            ),
        } satisfies echarts.CustomSeriesOption;
      } else {
        throw new Error("Unknown key type: " + JSON.stringify(key));
      }
    },
    [
      usagesPerApp,
      usagesPerAppSession,
      interactionPeriods,
      systemEvents,
      appInfoBarHeight,
      interval,
    ],
  );

  useEffect(() => {
    if (!chartInit) return;

    const timeGap = minRenderTimeGap(interval, width, dataZoom);
    const color = getVarColorAsHex("primary");

    // ---------------------------
    // Focus / Distractive Streaks
    // ---------------------------
    // Render background areas for the provided streaks using markArea. If a streak
    // is focused we render it in green, otherwise in red. We span the whole
    // vertical space of the chart so that it covers every bar.

    const streakMarkAreaData: echarts.MarkAreaComponentOption["data"] = (
      streaks ?? []
    )
      .filter((p) => p.start && p.end)
      .map((p) => [
        {
          coord: [ticksToUnixMillis(p.start), 0],
          // Individual color per streak
          itemStyle: {
            color: p.isFocused
              ? FOCUS_STREAK_BACKGROUND
              : DISTRACTIVE_STREAK_BACKGROUND,
          },
        },
        {
          coord: [ticksToUnixMillis(p.end), seriesHeight],
        },
      ]);

    const streakSeries: echarts.SeriesOption[] = streakMarkAreaData.length
      ? [
          {
            type: "scatter", // dummy series just to host markArea
            name: "streaks",
            data: [],
            silent: true,
            markArea: {
              silent: true,
              z: -2000,
              itemStyle: {
                opacity: 0.3,
              },
              data: streakMarkAreaData,
            },
          } as echarts.ScatterSeriesOption,
        ]
      : [];

    const series = [
      ...streakSeries,
      ...seriesKeys.map((key) => seriesKeyToSeries(key, timeGap, color)),
    ];

    const startValue = getDataZoomYStartValue(chartInstanceRef.current!);

    const commonOptions = {
      grid: {
        left: infoGap,
        // yes, innerHeight is the max height of the grid
        // it's only seriesHeight when we have too few elements to render
        height: Math.min(innerHeight, seriesHeight),
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
      dataZoom: [
        {
          id: DATAZOOM_INSIDE_Y,
          startValue,
          endValue: startValue + innerHeight,
        },
      ],
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

    // Force recalculation of item sizes
    infoBarsRef.current?.resetAfterIndex(0);

    // Force resize now instead of waiting for the next frame
    topInstanceRef.current?.resize();
    chartInstanceRef.current?.resize();
  }, [
    interactionPeriods,
    systemEvents,
    seriesKeys,
    seriesKeyToSeries,
    width,
    interval,
    infoGap,
    seriesHeight,
    chartInit,
    dataZoom,
    innerHeight,
    streaks,
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
        height: innerHeight,
        right: scrollbarWidth,
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
        min: 0,
        max: innerHeight,
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
          id: DATAZOOM_SLIDER_X,
          type: "slider",
          xAxisIndex: [0],
          filterMode: "weakFilter",
          top: 5,
        },
        {
          id: DATAZOOM_INSIDE_X,
          type: "inside",
          xAxisIndex: [0],
          filterMode: "weakFilter",
          // zoomOnMouseWheel: "shift",
          // moveOnMouseWheel: false,
          // preventDefaultMouseMove: false,
        },
        {
          id: DATAZOOM_INSIDE_Y,
          type: "inside",
          yAxisIndex: [0],
          filterMode: "weakFilter",
          zoomLock: true,
          zoomOnMouseWheel: false,
          startValue: 0,
          endValue: innerHeight,
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
        formatter: () => "",
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
        {
          ...common.dataZoom[2],
        },
      ],
    };

    chart.setOption(option);
    top.setOption(optionTop);
    echarts.connect([chart, top]);

    chart.on("datazoom", (arg0) => {
      const params = arg0 as DataZoomEvent;
      const batch = params.batch ? params.batch : [params];

      for (const params of batch) {
        if (
          params.id === DATAZOOM_INSIDE_Y ||
          params.dataZoomId === DATAZOOM_INSIDE_Y
        ) {
          // Y-axis changed
          const startValue = params.startValue ?? getDataZoomYStartValue(chart);
          infoBarsRef.current!.scrollTo(startValue);
          // if scrollbar changed which changed datazoom,
          // we update the scrollbar again. a bit of waste
          // (and you can see the jankiness when we scroll near 100%) but meh
          scrollbarRef.current!.scrollTop = startValue;
        } else {
          // X-axis changed

          // percentage (100)
          const diff = params.end - params.start;
          setDataZoom(diff);
        }
      }
    });

    chart.getZr().on("mousemove", () => {
      setHoverData(null);
      setHoverInteraction(null);
      setHoverSystemEvent(null);
    });

    chart.on("mousemove", (params) => {
      const [
        ,
        ,
        startMillis,
        endMillis,
        typeI,
        ,
        boolI,
        count,
        entityId,
        sessionId,
        appId,
      ] = params.data as number[];
      const start = unixMillisToTicks(startMillis);
      const end = unixMillisToTicks(endMillis);

      const type = typeI as unknown as SessionSeriesKey["type"];
      if (type === "session" || type === "app") {
        let usage: UsageBar | null = null;
        if (count > 1) {
          usage = {
            type: "combined",
            start,
            end,
            count,
          };
        } else {
          usage = {
            start,
            end,
            sessionId: +sessionId as Ref<Session>,
            appId: +appId as Ref<App>,
          };
        }
        setHoverData(usage);
        setHoverInteraction(null);
        setHoverSystemEvent(null);
      } else if (type === "interactionBar") {
        let interaction: InteractionBar | null = null;
        const isSystemEvent = boolI;
        if (isSystemEvent) {
          setHoverSystemEvent({
            id: +entityId as Ref<SystemEvent>,
            timestamp: start,
            event: sessionId as unknown as SystemEventEnum,
          });
          setHoverData(null);
          setHoverInteraction(null);
        } else {
          const mouseClicks = sessionId;
          const keyStrokes = appId;

          if (count > 1) {
            interaction = {
              type: "combined",
              start,
              end,
              count,
              mouseClicks,
              keyStrokes,
            };
          } else {
            interaction = {
              start,
              end,
              id: +entityId as Ref<InteractionPeriod>,
              mouseClicks,
              keyStrokes,
            };
          }
          setHoverInteraction(interaction);
          setHoverData(null);
          setHoverSystemEvent(null);
        }
      } else {
        throw new Error("Unknown type: " + JSON.stringify(params.data));
      }
    });

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
  }, [setDataZoom, innerHeight, scrollbarWidth]);

  const handleScrollbarScroll = useCallback(
    (event: React.UIEvent<HTMLDivElement, UIEvent>) => {
      const startValue = event.currentTarget.scrollTop;
      setDataZoomYWindow(
        chartInstanceRef.current!,
        startValue,
        startValue + innerHeight,
      );
      // infoBarsRef.current!.scrollTop = startValue;
    },
    [innerHeight],
  );

  const ListItem = useCallback(
    ({ index, style }: { index: number; style: CSSProperties }) => {
      const key = seriesKeys[index];

      return (
        <div style={style}>
          {key.type === "interactionBar" ? (
            <InteractionInfoBar
              loading={
                (interactionPeriodsLoading ?? false) ||
                (systemEventsLoading ?? false) ||
                (streaksLoading ?? false)
              }
              validating={
                (interactionPeriodsValidating ?? false) ||
                (systemEventsValidating ?? false) ||
                (streaksValidating ?? false)
              }
              interactionInfoBarHeight={interactionInfoBarHeight}
              key={key.type + key.id}
            />
          ) : key.type === "app" ? (
            <AppInfoBar
              appInfoBarHeight={appInfoBarHeight}
              key={key.type + key.id}
              // className="relative"
              onClick={() => toggleApp(key.id)}
              expanded={expanded[key.id]}
              app={appMap[key.id]!}
            />
          ) : (
            <SessionInfoBar
              session={usagesPerAppSession[key.appId][key.id]}
              sessionInfoBarHeight={sessionInfoBarHeight}
              sessionInfoUrlBarHeight={sessionInfoUrlBarHeight}
              // className="relative"
              key={key.type + key.id}
            />
          )}
        </div>
      );
    },
    [
      seriesKeys,
      usagesPerAppSession,
      sessionInfoBarHeight,
      sessionInfoUrlBarHeight,
      appInfoBarHeight,
      interactionInfoBarHeight,
      interactionPeriodsLoading,
      systemEventsLoading,
      streaksLoading,
      interactionPeriodsValidating,
      systemEventsValidating,
      streaksValidating,
      toggleApp,
      expanded,
      appMap,
    ],
  );

  const listItemSize = useCallback(
    (index: number) => {
      const key = seriesKeys[index];
      return getItemHeight(
        key,
        interactionInfoBarHeight,
        appInfoBarHeight,
        sessionInfoBarHeight,
        sessionInfoUrlBarHeight,
        key.type === "session"
          ? usagesPerAppSession[key.appId][key.id]
          : undefined,
      );
    },
    [
      seriesKeys,
      interactionInfoBarHeight,
      appInfoBarHeight,
      sessionInfoBarHeight,
      sessionInfoUrlBarHeight,
      usagesPerAppSession,
    ],
  );

  // if loading or no apps, show innerHeight to show the empty states
  // else, fit to content size
  const innerRelevantHeight =
    usagesLoading || apps.length === 0
      ? innerHeight
      : Math.min(innerHeight, seriesHeight);

  return (
    <div className="w-full h-full sticky">
      {/* Top bar */}
      <div
        className="sticky z-10 w-full border-border border-b bg-card top-0 shadow-md dark:shadow-xl"
        style={{ height: 90 }}
      >
        <div ref={topRef} className="w-full h-full"></div>

        <div
          className="absolute top-0 left-0 bottom-0"
          style={{ width: infoGap }}
        >
          {summary}
        </div>
      </div>

      {/* Main Content */}
      <div className="relative">
        <div
          ref={chartRef}
          className="w-full"
          style={{
            height: innerRelevantHeight,
          }}
        >
          <VizTooltip show={hoverData !== null} targetRef={chartRef}>
            <div className="max-w-[800px]">
              {(hoverData as CombinedUsage)?.type === "combined" ? (
                <CombinedUsageTooltip usage={hoverData as CombinedUsage} />
              ) : (
                <AppSessionUsageTooltip
                  usage={hoverData as AppSessionUsage}
                  usagesPerAppSession={usagesPerAppSession}
                />
              )}
            </div>
          </VizTooltip>

          <VizTooltip show={hoverInteraction !== null} targetRef={chartRef}>
            <div className="max-w-[800px]">
              {(hoverInteraction as CombinedInteractionPeriod)?.type ===
              "combined" ? (
                <CombinedInteractionTooltip
                  interaction={hoverInteraction as CombinedInteractionPeriod}
                />
              ) : (
                <InteractionTooltip
                  interaction={hoverInteraction as InteractionPeriod}
                />
              )}
            </div>
          </VizTooltip>

          <VizTooltip show={hoverSystemEvent !== null} targetRef={chartRef}>
            <div className="max-w-[800px]">
              {hoverSystemEvent && (
                <div className={cn("flex flex-col")}>
                  <div className="flex items-center gap-1 text-sm">
                    <Text className="text-base">
                      {systemEventToString(hoverSystemEvent.event)}
                    </Text>
                  </div>
                  <div className="flex items-center text-muted-foreground gap-1 text-xs">
                    <DateTimeText ticks={hoverSystemEvent.timestamp} />
                  </div>
                </div>
              )}
            </div>
          </VizTooltip>
        </div>
        {/* Info Bars */}
        <div
          className="absolute top-0 left-0 bottom-0"
          style={{ width: infoGap }}
        >
          <List
            ref={infoBarsRef}
            height={innerRelevantHeight}
            width={infoGap}
            itemCount={seriesKeys.length}
            style={{ overflow: "hidden" }} // hide scrollbar
            itemSize={listItemSize}
          >
            {ListItem}
          </List>
        </div>
        {/* Scrollbar */}
        <div
          ref={scrollbarRef}
          className="absolute top-0 right-0 bottom-0 overflow-y-auto"
          style={{ height: innerRelevantHeight, width: scrollbarWidth }}
          onScroll={handleScrollbarScroll}
        >
          <div className="w-px" style={{ height: seriesHeight }} />
        </div>

        {/* Session Empty State Indicator */}
        {!usagesLoading && apps.length === 0 && (
          <div className="absolute inset-0">
            <div className="flex flex-col items-center justify-center p-8 text-muted-foreground h-full">
              <LaptopIcon className="w-12 h-12 mb-4" />
              <Text className="text-lg font-semibold mb-2">No Activity</Text>
              <Text className="text-sm">
                No application usage data available for this time period
              </Text>
            </div>
          </div>
        )}

        {/* Session Loading Indicator */}
        {usagesLoading && (
          <div className="absolute inset-0">
            <div className="flex flex-col items-center justify-center p-8 text-muted-foreground h-full">
              <LaptopIcon className="w-12 h-12 mb-4 animate-pulse" />
              <Text className="text-lg font-semibold mb-2">Loading...</Text>
              <Text className="text-sm animate-pulse">
                Fetching application usage data
              </Text>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function InteractionInfoBar({
  loading,
  validating,
  interactionInfoBarHeight,
  className,
}: {
  loading: boolean;
  validating: boolean;
  interactionInfoBarHeight: number;
  className?: ClassValue;
}) {
  return (
    <div className={cn(className)}>
      <div
        className="flex items-center p-4 border-r"
        style={{ height: interactionInfoBarHeight }}
      >
        <LaptopIcon className="w-6 h-6 ml-6" />
        <Text className="font-semibold ml-4">Interactions</Text>
        {(loading || validating) && (
          <Loader2Icon className="animate-spin ml-4" />
        )}
      </div>
      <div className="h-px bg-border absolute bottom-[-0.5px] left-0 right-0" />
    </div>
  );
}

function AppInfoBar({
  app,
  appInfoBarHeight,
  className,
  onClick,
  expanded,
}: {
  app: App;
  appInfoBarHeight: number;
  className?: ClassValue;
  onClick: () => void;
  expanded: boolean;
}) {
  return (
    <div className={cn(className)}>
      <div
        className="flex items-center p-4 bg-muted/80 hover:bg-muted/60 border-r"
        style={{ height: appInfoBarHeight }}
        onClick={onClick}
      >
        {expanded ? (
          <ChevronDown size={20} className="flex-shrink-0" />
        ) : (
          <ChevronRight size={20} className="flex-shrink-0" />
        )}
        <AppIcon appIcon={app.icon} className="ml-2 w-6 h-6 shrink-0" />
        <Text className="font-semibold ml-4">{app.name}</Text>
      </div>
      <div className="h-px bg-border absolute bottom-[-0.5px] left-0 right-0" />
    </div>
  );
}

function SessionInfoBar({
  session,
  sessionInfoBarHeight,
  sessionInfoUrlBarHeight,
  className,
}: {
  session: Session;
  sessionInfoBarHeight: number;
  sessionInfoUrlBarHeight: number;
  className?: ClassValue;
}) {
  return (
    <div className={cn(className)}>
      <div
        className="p-4 border-t border-r flex flex-col justify-center"
        style={{
          height: session.url ? sessionInfoUrlBarHeight : sessionInfoBarHeight,
        }}
      >
        <Text className="text-sm">{session.title}</Text>
        {session.url && (
          <Text className="text-xs font-mono text-muted-foreground">
            {session.url}
          </Text>
        )}
        <div className="text-xs text-muted-foreground inline-flex gap-1 items-center">
          <DateTimeText ticks={session.start} />
          <span className="text-muted-foreground">-</span>
          <DateTimeText ticks={session.end} className="text-xs" />
        </div>
      </div>
      <div className="h-px bg-border absolute bottom-[-0.5px] left-0 right-0" />
    </div>
  );
}

function CombinedUsageTooltip({ usage }: { usage: CombinedUsage }) {
  return (
    <div className="flex flex-col">
      <Text
        children={`${usage.count} Combined Usages. Zoom to see details.`}
      ></Text>
      <DateTimeRange start={usage.start} end={usage.end} />
    </div>
  );
}

function AppSessionUsageTooltip({
  usage,
  usagesPerAppSession,
}: {
  usage: AppSessionUsage;
  usagesPerAppSession: AppSessionUsages;
}) {
  const session = usagesPerAppSession[usage.appId][usage.sessionId];
  return (
    <div className="flex flex-col">
      <div className="flex flex-col">
        <Text>{session.title}</Text>
        {session.url && <Text className="text-xs">{session.url}</Text>}
        <DateTimeRange start={usage.start} end={usage.end} />

        <Text className="border-t mt-2 border-border" children={`Usage`}></Text>
        <DateTimeRange start={usage.start} end={usage.end} />
      </div>
    </div>
  );
}

function CombinedInteractionTooltip({
  interaction,
}: {
  interaction: CombinedInteractionPeriod;
}) {
  return (
    interaction && (
      <div className={cn("flex flex-col")}>
        <div className="flex items-center gap-1 text-sm">
          <Text
            className="text-base"
            children={`${interaction.count} Combined Interactions. Zoom to see details.`}
          ></Text>
          <KeyboardIcon size={16} className="ml-4" />
          <div>{interaction.keyStrokes}</div>
          <MouseIcon size={16} className="ml-2" />
          <div>{interaction.mouseClicks}</div>
        </div>
        <DateTimeRange start={interaction.start} end={interaction.end} />
      </div>
    )
  );
}

function InteractionTooltip({
  interaction,
}: {
  interaction: InteractionPeriod;
}) {
  return (
    interaction && (
      <div className={cn("flex flex-col")}>
        <div className="flex items-center gap-1 text-sm">
          <Text className="text-base">Interaction</Text>
          <KeyboardIcon size={16} className="ml-4" />
          <div>{interaction.keyStrokes}</div>
          <MouseIcon size={16} className="ml-2" />
          <div>{interaction.mouseClicks}</div>
        </div>
        <DateTimeRange start={interaction.start} end={interaction.end} />
      </div>
    )
  );
}

export function DurationSummaries({
  usages,
  usagesLoading,
  usagesValidating,

  streaks,
  streaksLoading,
  streaksValidating,

  className,
}: {
  usages: AppSessionUsages;
  usagesLoading?: boolean;
  usagesValidating?: boolean;

  streaks?: Streak[];
  streaksLoading?: boolean;
  streaksValidating?: boolean;

  className?: ClassValue;
}) {
  const [focusStreakUsage, distractiveStreakUsage] =
    useStreakDurations(streaks);

  const totalUsage = useMemo(() => {
    return _(usages)
      .flatMap((sessions) => Object.values(sessions))
      .flatMap((session) =>
        Object.values(session.usages).map((usage) => usage.end - usage.start),
      )
      .sum();
  }, [usages]);

  return (
    <div
      className={cn(
        "flex items-center gap-1 text-muted-foreground text-lg font-semibold",
        className,
      )}
    >
      {streaks &&
        (streaksLoading ? (
          <>
            <Loader2Icon className="animate-spin" />
            <div>/</div>
            <Loader2Icon className="animate-spin" />
          </>
        ) : (
          <>
            <DurationText
              ticks={focusStreakUsage}
              className="text-green-500/80"
              description="Focus Streaks"
            />
            {streaksValidating && <Loader2Icon className="animate-spin" />}
            <div>/</div>
            <DurationText
              ticks={distractiveStreakUsage}
              className="text-red-400/80"
              description="Distractive Streaks"
            />
            {streaksValidating && <Loader2Icon className="animate-spin" />}
          </>
        ))}
      {streaks && <div>/</div>}
      {usagesLoading ? (
        <Loader2Icon className="animate-spin" />
      ) : (
        <>
          <DurationText
            ticks={totalUsage}
            className="text-foreground/80"
            description="Total Usage"
          />
          {usagesValidating && <Loader2Icon className="animate-spin" />}
        </>
      )}
    </div>
  );
}

function DateTimeRange({ start, end }: { start: number; end: number }) {
  return (
    <div className="flex items-center text-muted-foreground gap-1 text-xs">
      <DateTimeText ticks={start} /> -
      <DateTimeText ticks={end} />
      (<DurationText ticks={end - start} />)
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
  const minRenderTimeGap = minRenderTimeGapMillis * TICKS_PER_MILLISECOND;
  return Math.max(minRenderTimeGap, 1);
}

export function mergedUsages(
  usages: AppSessionUsage[],
  minRenderTimeGap: number,
): UsageBar[] {
  if (minRenderTimeGap < maxRenderTimeGap) {
    return usages;
  }
  const mergedUsages: UsageBar[] = [{ ...usages[0] }];
  for (let i = 1; i < usages.length; i++) {
    const usage = usages[i];
    const lastUsage = mergedUsages[mergedUsages.length - 1];
    if (lastUsage.end + minRenderTimeGap > usage.start) {
      let lastUsageBar = lastUsage as CombinedUsage;
      if (lastUsageBar.type !== "combined") {
        lastUsageBar = mergedUsages[mergedUsages.length - 1] = {
          start: lastUsage.start,
          end: lastUsage.end,
          type: "combined",
          count: 1,
        };
      }
      lastUsageBar.end = usage.end;
      lastUsageBar.count += 1;
    } else {
      mergedUsages.push({ ...usage });
    }
  }
  return mergedUsages;
}

export function mergedInteractionPeriods(
  interactionPeriods: InteractionPeriod[],
  minRenderTimeGap: number,
): InteractionBar[] {
  if (minRenderTimeGap < maxRenderTimeGap) {
    return interactionPeriods;
  }
  const mergedInteractionPeriods: InteractionBar[] = [
    { ...interactionPeriods[0] },
  ];
  for (let i = 1; i < interactionPeriods.length; i++) {
    const interactionPeriod = interactionPeriods[i];

    const lastInteractionPeriod =
      mergedInteractionPeriods[mergedInteractionPeriods.length - 1];
    if (
      lastInteractionPeriod.end + minRenderTimeGap >
      interactionPeriod.start
    ) {
      let lastInteractionPeriodBar =
        lastInteractionPeriod as CombinedInteractionPeriod;
      if (lastInteractionPeriodBar.type !== "combined") {
        lastInteractionPeriodBar = mergedInteractionPeriods[
          mergedInteractionPeriods.length - 1
        ] = {
          start: lastInteractionPeriod.start,
          end: lastInteractionPeriod.end,
          mouseClicks: lastInteractionPeriod.mouseClicks,
          keyStrokes: lastInteractionPeriod.keyStrokes,
          type: "combined",
          count: 1,
        };
      }
      lastInteractionPeriodBar.end = interactionPeriod.end;
      lastInteractionPeriodBar.mouseClicks += interactionPeriod.mouseClicks;
      lastInteractionPeriodBar.keyStrokes += interactionPeriod.keyStrokes;
      lastInteractionPeriodBar.count += 1;
    } else {
      mergedInteractionPeriods.push({ ...interactionPeriod });
    }
  }
  return mergedInteractionPeriods;
}

function appBar(
  color: string,
  top: number,
  bottom: number,
  interval: Interval,
  backgroundColor: string,
): echarts.CustomSeriesOption {
  return {
    animation: false,
    type: "custom",
    progressive: 0,
    markArea: {
      silent: true,
      z: -1000,
      zlevel: -1000,
      itemStyle: {
        color: backgroundColor,
        opacity: 0.5,
      },
      data: [
        [
          { coord: [interval.start.toMillis(), top] },
          { coord: [interval.end.toMillis(), bottom] },
        ],
      ],
    },
    renderItem: (
      params: echarts.CustomSeriesRenderItemParams,
      api: echarts.CustomSeriesRenderItemAPI,
    ): echarts.CustomSeriesRenderItemReturn => {
      const top = api.value(0);
      const bottom = api.value(1);
      const start = api.value(2);
      const end = api.value(3);

      const [x0, y0] = api.coord([start, top]);
      const [x1, y1] = api.coord([end, bottom]);
      // minimum 1px width
      const width = Math.max(x1 - x0, 1);
      const height = y1 - y0;

      const rectShape = {
        x: x0,
        y: y0,
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

function seriesBar(color: string): echarts.CustomSeriesOption {
  return {
    animation: false,
    type: "custom",
    progressive: 0,
    renderItem: (
      params: echarts.CustomSeriesRenderItemParams,
      api: echarts.CustomSeriesRenderItemAPI,
    ): echarts.CustomSeriesRenderItemReturn => {
      const top = api.value(0);
      const bottom = api.value(1);
      const start = api.value(2);
      const end = api.value(3);

      const [x0, y0] = api.coord([start, top]);
      const [x1, y1] = api.coord([end, bottom]);
      // minimum 1px width
      const width = Math.max(x1 - x0, 1);
      const height = y1 - y0;

      const rectShape = {
        x: x0,
        y: y0,
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

function interactionBar(
  color: string,
  systemEventColor: string,
): echarts.CustomSeriesOption {
  return {
    animation: false,
    type: "custom",
    progressive: 0,
    renderItem: (
      params: echarts.CustomSeriesRenderItemParams,
      api: echarts.CustomSeriesRenderItemAPI,
    ): echarts.CustomSeriesRenderItemReturn => {
      const top = api.value(0);
      const bottom = api.value(1);
      const start = api.value(2);
      const end = api.value(3);
      const isSystemEvent = api.value(6);

      const [x0, y0] = api.coord([start, top]);
      const [x1, y1] = api.coord([end, bottom]);
      // minimum 1px width
      const width = Math.max(x1 - x0, 1);
      const height = y1 - y0;
      const extra = 6;

      // EW manual coding of constants
      const rectShape = {
        x: x0,
        y: y0 - (isSystemEvent ? extra : 0),
        width,
        height: height + (isSystemEvent ? 2 * extra : 0),
      };
      const shape = echarts.graphic.clipRectByRect(
        rectShape,
        params.coordSys as unknown as RectLike,
      );

      return (
        shape && {
          type: "rect" as const,
          zlevel: isSystemEvent ? 0 : 1,
          z: isSystemEvent ? 0 : 1,
          z2: isSystemEvent ? 0 : 1,
          shape,
          style: {
            fill: isSystemEvent ? systemEventColor : color,
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
  hasInteractionInfoBar: boolean,
  interactionInfoBarHeight: number,
  appInfoBarHeight: number,
  sessionInfoBarHeight: number,
  sessionInfoUrlBarHeight: number,
  barHeight: number,
): [SeriesKey[], number] {
  const seriesKeys: SeriesKey[] = [];
  let y = 0;
  if (hasInteractionInfoBar) {
    seriesKeys.push({
      type: "interactionBar",
      id: 0,
      y,
      top: y + interactionInfoBarHeight / 2 - barHeight / 2,
      bottom: y + interactionInfoBarHeight / 2 + barHeight / 2,
    });
    y += interactionInfoBarHeight;
  }
  for (const app of apps) {
    seriesKeys.push({
      type: "app",
      id: app.id,
      y,
      top: y + appInfoBarHeight / 2 - barHeight / 2,
      bottom: y + appInfoBarHeight / 2 + barHeight / 2,
    });
    y += appInfoBarHeight;

    if (expanded[app.id]) {
      for (const sessionId of Object.keys(usagesPerAppSession[app.id])) {
        const session = +sessionId as Ref<Session>;
        const relevantHeight = usagesPerAppSession[app.id][session].url
          ? sessionInfoUrlBarHeight
          : sessionInfoBarHeight;
        seriesKeys.push({
          type: "session",
          id: session,
          appId: app.id,
          y,
          top: y + relevantHeight / 2 - barHeight / 2,
          bottom: y + relevantHeight / 2 + barHeight / 2,
        });
        y += relevantHeight;
      }
    }
  }
  return [seriesKeys, y];
}

function getDataZoomYStartValue(chart: echarts.ECharts): number {
  return (
    ((
      (chart?.getOption() as echarts.EChartsOption | null)
        ?.dataZoom as echarts.DataZoomComponentOption[]
    )?.[2]?.startValue as number) ?? 0
  );
}

function setDataZoomYWindow(
  chart: echarts.ECharts,
  startValue: number,
  endValue: number,
): void {
  chart.dispatchAction({
    type: "dataZoom",
    dataZoomId: DATAZOOM_INSIDE_Y,
    startValue,
    endValue,
  });
}

function getItemHeight(
  key: SeriesKey,
  interactionInfoBarHeight: number,
  appInfoBarHeight: number,
  sessionInfoBarHeight: number,
  sessionInfoUrlBarHeight: number,
  session?: Session,
): number {
  if (key.type === "app") {
    return appInfoBarHeight;
  } else if (key.type === "session") {
    // For session items, we need to check if it has a URL to determine height
    return session?.url ? sessionInfoUrlBarHeight : sessionInfoBarHeight;
  } else if (key.type === "interactionBar") {
    return interactionInfoBarHeight;
  }
  throw new Error("Unknown key type: " + JSON.stringify(key));
}

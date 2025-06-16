import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as echarts from "echarts";
import {
  systemEventToString,
  type App,
  type Ref,
  type Session,
  type SystemEventEnum,
  type Usage,
} from "@/lib/entities";
import type { InteractionPeriod, SystemEvent } from "@/lib/entities";
import type { AppSessionUsages } from "@/lib/repo";
import {
  ticksToUnixMillis,
  unixMillisToTicks,
  type Interval,
} from "@/lib/time";
import { useWidth } from "@/hooks/use-width";
import _ from "lodash";
import { useApps } from "@/hooks/use-refresh";
import {
  ChevronDown,
  ChevronRight,
  KeyboardIcon,
  LaptopIcon,
  MouseIcon,
} from "lucide-react";
import AppIcon from "@/components/app/app-icon";
import { Text } from "@/components/ui/text";
import { getVarColorAsHex } from "@/lib/color-utils";
import { useDebounce } from "use-debounce";
import { useAppState } from "@/lib/state";
import { DateTimeText } from "@/components/time/time-text";
import { Tooltip } from "@/components/viz/tooltip";
import { DurationText } from "@/components/time/duration-text";
import type { ClassValue } from "clsx";
import { cn } from "@/lib/utils";
import { VariableSizeList as List } from "react-window";

const DATAZOOM_SLIDER_X = "dataZoomSliderX";
const DATAZOOM_INSIDE_X = "dataZoomInsideX";
const DATAZOOM_INSIDE_Y = "dataZoomInsideY";

type RectLike = {
  x: number;
  y: number;
  width: number;
  height: number;
};

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
  usageId: Ref<Usage>;
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

  interactionPeriods?: InteractionPeriod[];
  interactionPeriodsLoading?: boolean;

  systemEvents?: SystemEvent[];
  systemEventsLoading?: boolean;

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

export function Gantt2({
  usages: usagesPerAppSession,
  // usagesLoading,
  interactionPeriods,
  // interactionPeriodsLoading,
  systemEvents,
  // systemEventsLoading,
  defaultExpanded,
  interval,
  infoGap = 300,
  interactionInfoBarHeight = 52,
  appInfoBarHeight = 52,
  sessionInfoBarHeight = 72,
  sessionInfoUrlBarHeight = 84,
  innerHeight = 500,
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

  const appIds = useMemo(() => {
    return Object.keys(usagesPerAppSession).map((appId) => +appId as Ref<App>);
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
                    usageId: usage.id,
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
        !!(interactionPeriods || systemEvents),
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
      const id = key.type + key.id;
      if (key.type === "app") {
        const usages = mergedUsages(usagesPerApp[key.id], timeGap);
        return {
          ...appBar(color),
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
            (usage as AppSessionUsage).usageId,
            (usage as AppSessionUsage).sessionId,
            (usage as AppSessionUsage).appId,
          ]),
        } satisfies echarts.CustomSeriesOption;
      } else if (key.type === "session") {
        const usages = mergedUsages(
          usagesPerAppSession[key.appId][key.id].usages.map((usage) => ({
            start: usage.start,
            end: usage.end,
            usageId: usage.id,
            sessionId: key.id,
            appId: key.appId,
          })),
          timeGap,
        );
        return {
          ...appBar(color),
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
            (usage as AppSessionUsage).usageId,
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
          ...interactionBar(color, "orange"),
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
    [usagesPerApp, usagesPerAppSession, interactionPeriods, systemEvents],
  );

  useEffect(() => {
    if (!chartInit) return;

    const timeGap = minRenderTimeGap(interval, width, dataZoom);
    const color = getVarColorAsHex("primary");
    const series = seriesKeys.map((key) =>
      seriesKeyToSeries(key, timeGap, color),
    );

    const startValue = getDataZoomYStartValue(chartInstanceRef.current!);

    const commonOptions = {
      grid: {
        left: infoGap,
        height: innerHeight,
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
          const startValue = getDataZoomYStartValue(chart);
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
        usageId,
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
            usageId: +usageId as Ref<Usage>,
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
            id: +usageId as Ref<SystemEvent>,
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
              id: +usageId as Ref<InteractionPeriod>,
              mouseClicks,
              keyStrokes,
            };
          }
          setHoverInteraction(interaction);
          setHoverData(null);
          setHoverSystemEvent(null);
        }
      } else {
        throw new Error("Unknown type: " + type);
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

  return (
    <div className="w-full h-full sticky">
      {/* Top bar */}
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
            height: innerHeight,
          }}
        >
          <Tooltip show={hoverData !== null} targetRef={chartRef}>
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
          </Tooltip>

          <Tooltip show={hoverInteraction !== null} targetRef={chartRef}>
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
          </Tooltip>

          <Tooltip show={hoverSystemEvent !== null} targetRef={chartRef}>
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
          </Tooltip>
        </div>
        {/* Info Bars */}
        <div
          className="absolute top-0 left-0 bottom-0"
          style={{ width: infoGap }}
        >
          <List
            ref={infoBarsRef}
            height={innerHeight}
            width={infoGap}
            itemCount={seriesKeys.length}
            style={{ overflow: "hidden" }} // hide scrollbar
            itemSize={(index) => {
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
            }}
          >
            {({ index, style }) => {
              const key = seriesKeys[index];
              return (
                <div style={style}>
                  {key.type === "interactionBar" ? (
                    <InteractionInfoBar
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
            }}
          </List>
        </div>
        {/* Scrollbar */}
        <div
          ref={scrollbarRef}
          className="absolute top-0 right-0 bottom-0 overflow-y-auto"
          style={{ height: innerHeight, width: scrollbarWidth }}
          onScroll={handleScrollbarScroll}
        >
          <div className="w-px" style={{ height: seriesHeight }} />
        </div>
      </div>
    </div>
  );
}

function InteractionInfoBar({
  interactionInfoBarHeight,
  className,
}: {
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
        {/* BUG: this loading is as long as the usage loading ..?????
                {(interactionPeriodsLoading || systemEventsLoading) && (
                  <Loader2Icon className="animate-spin ml-4" />
                )} */}
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
        <AppIcon buffer={app.icon} className="ml-2 w-6 h-6 shrink-0" />
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
        className="text-muted-foreground"
        children={`${usage.count} Combined Usages. Zoom to see details.`}
      ></Text>
      <div className="flex items-center text-muted-foreground gap-1 text-xs">
        <DateTimeText ticks={usage.start} /> -
        <DateTimeText ticks={usage.end} />
        (
        <DurationText ticks={usage.end - usage.start} />)
      </div>
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
        {session.url && (
          <Text className="text-muted-foreground text-xs">{session.url}</Text>
        )}
        <div className="flex items-center text-muted-foreground gap-1 text-xs">
          <DateTimeText ticks={session.start} /> -
          <DateTimeText ticks={session.end} />
          (
          <DurationText ticks={session.end - session.start} />)
        </div>

        <Text
          className="text-muted-foreground border-t mt-2 border-border"
          children={`Usage: `}
        ></Text>
        <div className="flex items-center text-muted-foreground gap-1 text-xs">
          <DateTimeText ticks={usage.start} /> -
          <DateTimeText ticks={usage.end} />
          (
          <DurationText ticks={usage.end - usage.start} />)
        </div>
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
        <div className="flex items-center text-muted-foreground gap-1 text-xs">
          <DateTimeText ticks={interaction.start} /> -
          <DateTimeText ticks={interaction.end} />
          (
          <DurationText ticks={interaction.end - interaction.start} />)
        </div>
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
        <div className="flex items-center text-muted-foreground gap-1 text-xs">
          <DateTimeText ticks={interaction.start} /> -
          <DateTimeText ticks={interaction.end} />
          (
          <DurationText ticks={interaction.end - interaction.start} />)
        </div>
      </div>
    )
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
  usages: AppSessionUsage[],
  minRenderTimeGap: number,
): UsageBar[] {
  if (minRenderTimeGap < maxRenderTimeGap) {
    return usages;
  }
  const mergedUsages: UsageBar[] = [{ ...usages[0] }];
  for (const usage of usages) {
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
  for (const interactionPeriod of interactionPeriods) {
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

function appBar(color: string): echarts.CustomSeriesOption {
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

      // EW manual coding of constants
      const rectShape = {
        x: x0,
        y: y0 - (isSystemEvent ? 4 : 0),
        width,
        height: height + (isSystemEvent ? 2 * 4 : 0),
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
    )[2]?.startValue as number) ?? 0
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

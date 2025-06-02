import { useEffect, useRef } from "react";
import * as echarts from "echarts";
import type { App, Ref, Session } from "@/lib/entities";
import type { InteractionPeriod, SystemEvent } from "@/lib/entities";
import type { AppSessionUsages } from "@/lib/repo";
import { useApps } from "@/hooks/use-refresh";
import { ticksToDateTime, type Interval } from "@/lib/time";

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
  const apps = useApps(Object.keys(usages).map((id) => +id as Ref<App>));

  useEffect(() => {
    if (!chartRef.current) return;

    const chart = echarts.init(chartRef.current);
    chartInstanceRef.current = chart;

    const sessions: Session[] = Object.entries(usages).map(
      ([appId, sessions]) => {
        const usages = Object.values(sessions).flatMap(
          (session) => session.usages,
        );
        usages.sort((a, b) => a.start - b.start);
        return {
          id: +appId as Ref<Session>,
          title:
            apps.find((app) => app.id === +appId)?.name.substring(0, 10) ??
            "Unknown",
          start: usages[0].start,
          end: usages[usages.length - 1].end,
          usages: usages,
        };
      },
    );

    // Create series data for each session
    const seriesData: echarts.CustomSeriesOption[] = sessions.map(
      (session, index) => {
        return {
          name: session.title,
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
            const rowHeight = api.coord([0, index + 1])[1] - y[1];

            const x = api.coord([start, 0])[0];
            // minimum 1px width
            const width = Math.max(api.coord([end, 0])[0] - x, 1);

            const padding = 0.2;
            return {
              type: "rect" as const,
              shape: {
                x,
                y: y[1] - rowHeight * 0.5 + rowHeight * padding,
                width,
                height: rowHeight * (1 - padding * 2),
              },
              style: {
                fill: "#1890ff",
              },
            };
          },
          data: session.usages.map((usage) => [
            ticksToDateTime(usage.start).toMillis(),
            ticksToDateTime(usage.end).toMillis(),
          ]),
        };
      },
    );

    const option: echarts.EChartsOption = {
      tooltip: {
        trigger: "item",
        formatter: (params: any) => {
          const session = sessions[params.seriesIndex];
          const usage = session.usages[params.dataIndex];
          return `${session.title}<br/>Start: ${ticksToDateTime(usage.start).toFormat("yyyy-MM-dd HH:mm:ss")}<br/>End: ${ticksToDateTime(usage.end).toFormat("yyyy-MM-dd HH:mm:ss")}`;
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
        data: sessions.map((session) => session.title),
        axisLabel: {
          interval: 0,
        },
      },
      dataZoom: [
        {
          type: "slider",
          xAxisIndex: [0, 1],
          filterMode: "filter",
          top: 0,
        },
        {
          type: "inside",
          xAxisIndex: [0, 1],
          filterMode: "filter",
        },
      ],
      series: seriesData,
    };

    chart.setOption(option);
    console.log(chart);

    const resizeObserver = new ResizeObserver(() => {
      requestAnimationFrame(() => chart.resize());
    });

    resizeObserver.observe(chartRef.current);

    return () => {
      chart.dispose();
      resizeObserver.disconnect();
    };
  }, [usages, apps, interval]);

  return (
    <div className="w-full h-full">
      <div ref={chartRef} className="w-full h-full" />
    </div>
  );
}

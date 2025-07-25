import { HScrollView } from "@/components/hscroll-view";
import { UsageTooltipContent } from "@/components/viz/usage-tooltip";
import { VizTooltip } from "@/components/viz/viz-tooltip";
import { scaleColor } from "@/lib/color-utils";
import type { App, Ref, Tag } from "@/lib/entities";
import { useAppState } from "@/lib/state";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { DateTime, Interval } from "luxon";
import type React from "react";
import { useMemo, useRef, useState } from "react";

function rotateArray<T>(arr: T[], n: number) {
  return arr.slice(n).concat(arr.slice(0, n));
}

export interface HeatmapProps {
  startDate: DateTime;
  data: Map<number, number>; // DateTime -> value
  emptyCellColorRgb?: string;
  fullCellColorRgb?: string;
  className?: ClassValue;
  innerClassName?: ClassValue;
  axisClassName?: ClassValue;
  firstDayOfMonthClassName?: ClassValue;
  // returns between 0 and 1
  scaling: (value: number) => number;
  appId?: Ref<App>;
  tagId?: Ref<Tag>;
}

export interface HeatmapData {
  date: DateTime;
  day: number;
  week: number;
  value: number;
}

const DAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];
const CELL_SIZE = 14;
const PADDING_X = 30;
const PADDING_Y = 15;

const Heatmap: React.FC<HeatmapProps> = ({
  startDate,
  data,
  className,
  innerClassName,
  axisClassName,
  firstDayOfMonthClassName,
  emptyCellColorRgb = "var(--muted)",
  fullCellColorRgb = "#00FF00",
  scaling,
  appId,
  tagId,
}) => {
  const [tooltipData, setTooltipData] = useState<{
    x: number;
    y: number;
    date: DateTime;
    value: number;
  } | null>(null);

  const startWeek = useMemo(() => startDate.startOf("week"), [startDate]);
  const ref = useRef<SVGSVGElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);

  const apps = useAppState((state) => state.apps);
  const tags = useAppState((state) => state.tags);

  const heatmapData = useMemo(() => {
    const result: HeatmapData[] = [];
    const endDate = startDate.plus({ years: 1 });
    const interval = Interval.fromDateTimes(startDate, endDate);

    for (const day of interval.splitBy({ days: 1 })) {
      const currentDate = day.start!;
      const adjustedDay = (currentDate.weekday - 1 + 7) % 7;
      const week = Math.floor(day.start!.diff(startWeek, "days").days / 7);

      result.push({
        date: currentDate,
        day: adjustedDay,
        week,
        value: data.get(+currentDate) ?? 0,
      });
    }

    return result;
  }, [startDate, startWeek, data]);

  const maxWeek = Math.max(...heatmapData.map((d) => d.week));
  const width = (maxWeek + 1) * CELL_SIZE + PADDING_X;
  const height = 7 * CELL_SIZE + PADDING_Y + 4; // added 10px at the bottom

  const renderCells = () => {
    return heatmapData.map((entry, index) => {
      const intensity = scaling(entry.value);
      const fill =
        entry.value === 0
          ? emptyCellColorRgb
          : scaleColor(fullCellColorRgb, intensity);
      const cellDate = entry.date;
      const isFirstDayOfMonth = cellDate.day === 1;

      return (
        <rect
          key={index}
          x={entry.week * CELL_SIZE + PADDING_X}
          y={entry.day * CELL_SIZE + PADDING_Y}
          width={CELL_SIZE - 2}
          height={CELL_SIZE - 2}
          fill={fill}
          className={cn(
            isFirstDayOfMonth &&
              cn("stroke-primary stroke-1", firstDayOfMonthClassName),
          )}
          rx={3}
          onMouseEnter={(e) => {
            const rect = e.currentTarget.getBoundingClientRect();
            const parent = ref.current!.getBoundingClientRect();
            setTooltipData({
              x: rect.left - parent.left,
              y: rect.top - parent.top,
              date: entry.date,
              value: entry.value,
            });
          }}
          onMouseLeave={() => setTooltipData(null)}
        />
      );
    });
  };

  const renderAxes = () => {
    const yAxis = rotateArray(DAYS, 1).map((day, index) => (
      <text
        key={`day-${index}`}
        x={PADDING_X - 10}
        y={index * CELL_SIZE + PADDING_Y + CELL_SIZE / 2}
        textAnchor="end"
        dominantBaseline="middle"
        className={cn("fill-muted-foreground/75", axisClassName)}
        fontSize={10}
      >
        {day}
      </text>
    ));

    const xAxis = MONTHS.map((month, index) => {
      const firstDayOfMonth = startDate.set({ month: index + 1, day: 1 });
      const week = Math.floor(firstDayOfMonth.diff(startWeek, "days").days / 7);
      return (
        <text
          key={`month-${index}`}
          x={week * CELL_SIZE + PADDING_X + CELL_SIZE / 2}
          y={PADDING_Y - 5}
          textAnchor="middle"
          fontSize={10}
          className={cn("fill-muted-foreground/75", axisClassName)}
        >
          {month}
        </text>
      );
    });

    return [...yAxis, ...xAxis];
  };

  return (
    <HScrollView className={cn("relative overflow-x-auto", className)}>
      <div
        ref={containerRef}
        className={cn(innerClassName)}
        style={{ aspectRatio: width / height }}
      >
        <svg
          width="100%"
          height="100%"
          viewBox={`0 0 ${width} ${height}`}
          preserveAspectRatio="none"
          ref={ref}
        >
          {renderCells()}
          {renderAxes()}
        </svg>
      </div>
      <VizTooltip targetRef={containerRef} show={tooltipData !== null}>
        <UsageTooltipContent
          at={tooltipData?.date ?? DateTime.fromSeconds(0)}
          showRows={false}
          hovered={
            (tooltipData?.value ?? 0) === 0
              ? undefined
              : appId
                ? {
                    key: "app" as const,
                    app: apps[appId]!,
                    duration: tooltipData?.value ?? 0,
                  }
                : {
                    key: "tag" as const,
                    tag: tags[tagId!]!,
                    duration: tooltipData?.value ?? 0,
                  }
          }
          maximum={1}
        />
      </VizTooltip>
    </HScrollView>
  );
};

export default Heatmap;

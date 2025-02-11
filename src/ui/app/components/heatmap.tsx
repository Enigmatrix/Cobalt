import type React from "react";
import { useMemo, useRef, useState } from "react";
import { DateTime, Interval } from "luxon";
import type { ClassValue } from "clsx";
import { cn } from "@/lib/utils";
import { DateTimeText } from "./time-text";
import { DurationText } from "./duration-text";

function rotateArray<T>(arr: T[], n: number) {
  return arr.slice(n).concat(arr.slice(0, n));
}

export interface HeatmapProps {
  startDate: DateTime;
  data: Map<number, number>; // DateTime -> value
  axisClassName?: ClassValue;
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
const PADDING = 25;

const Heatmap: React.FC<HeatmapProps> = ({
  startDate,
  data,
  axisClassName,
}) => {
  const [tooltipData, setTooltipData] = useState<{
    x: number;
    y: number;
    date: DateTime;
    value: number;
  } | null>(null);

  const startWeek = useMemo(() => startDate.startOf("week"), [startDate]);
  const ref = useRef<SVGSVGElement | null>(null);

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
        value: data.get(+currentDate) || 0,
      });
    }

    return result;
  }, [startDate, data]);

  const maxWeek = Math.max(...heatmapData.map((d) => d.week));
  const width = (maxWeek + 1) * CELL_SIZE + PADDING;
  const height = 7 * CELL_SIZE + PADDING + 10; // added 10px at the bottom

  const renderCells = () => {
    return heatmapData.map((entry, index) => {
      const intensity = entry.value / 360000000000 + 0.1; // 10 hours
      const fill =
        entry.value === 0 ? "#222222" : `rgba(0, 128, 0, ${intensity})`;
      const cellDate = entry.date;
      const isFirstDayOfMonth = cellDate.day === 1;

      return (
        <rect
          key={index}
          x={entry.week * CELL_SIZE + PADDING}
          y={entry.day * CELL_SIZE + PADDING}
          width={CELL_SIZE - 2}
          height={CELL_SIZE - 2}
          fill={fill}
          stroke={isFirstDayOfMonth ? "#000" : "none"}
          strokeWidth={isFirstDayOfMonth ? 2 : 0}
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
        x={PADDING - 5}
        y={index * CELL_SIZE + PADDING + CELL_SIZE / 2}
        textAnchor="end"
        dominantBaseline="middle"
        className={cn(axisClassName)}
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
          x={week * CELL_SIZE + PADDING + CELL_SIZE / 2}
          y={PADDING - 5}
          textAnchor="middle"
          fontSize={10}
          className={cn(axisClassName)}
        >
          {month}
        </text>
      );
    });

    return [...yAxis, ...xAxis];
  };

  return (
    <div className="relative overflow-x-auto">
      <div
        className="min-h-[200px] aspect-video"
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
      {tooltipData && (
        <div
          className="absolute bg-card p-2 border border-border rounded shadow text-sm pointer-events-none"
          style={{
            left: `${tooltipData.x + 10}px`,
            top: `${tooltipData.y + 10}px`,
          }}
        >
          <DateTimeText
            datetime={tooltipData.date}
            className="font-semibold whitespace-nowrap"
          />
          <DurationText ticks={tooltipData.value} />
        </div>
      )}
    </div>
  );
};

export default Heatmap;

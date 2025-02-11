import type { DateTime } from "luxon";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { DurationText } from "@/components/duration-text";
import { useToday } from "@/hooks/use-today";

export function UsageCard({
  start: originalStart,
  usage,
  totalUsage,
  endFn,
  nextFn,
  prevFn,
  titleFn,
  onChanged,
  children,
  isLoading,
}: {
  start: DateTime;
  usage: number;
  totalUsage: number;
  endFn: (start: DateTime) => DateTime;
  nextFn: (start: DateTime) => DateTime;
  prevFn: (start: DateTime) => DateTime;
  titleFn: (dt: DateTime) => string;
  onChanged: (arg: { start: DateTime; end: DateTime }) => void;
  children: React.ReactNode;
  isLoading: boolean;
}) {
  const [start, setStart] = useState(originalStart);

  const next = useCallback(() => {
    const newStart = nextFn(start);
    setStart(newStart);
    onChanged({ start: newStart, end: endFn(newStart) });
  }, [nextFn, start, setStart]);

  const prev = useCallback(() => {
    const newStart = prevFn(start);
    setStart(newStart);
    onChanged({ start: newStart, end: endFn(newStart) });
  }, [prevFn, start, setStart]);

  useEffect(() => {
    onChanged({ start, end: endFn(start) });
  }, []);

  return (
    <div className="flex flex-col rounded-xl bg-muted/50">
      <div className="flex items-center">
        <div className="flex flex-col p-4 pb-1 min-w-0">
          <div className="whitespace-nowrap text-base text-foreground/50">
            {titleFn(start)}
          </div>
          <div className="flex gap-2 items-baseline font-semibold">
            <DurationText className="text-xl" ticks={usage} />
            {usage !== 0 && (
              <>
                <span className="text-xl text-muted-foreground">/</span>
                <DurationText
                  className="text-muted-foreground"
                  ticks={totalUsage}
                />
              </>
            )}
          </div>
        </div>

        <div className="flex-1" />

        {/* hide button controls between md: and lg: */}
        <div className="flex m-2 max-lg:hidden max-md:block">
          <Button
            variant="ghost"
            size="icon"
            onClick={prev}
            disabled={isLoading}
          >
            <ChevronLeft />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={next}
            disabled={isLoading || +start === +originalStart}
          >
            <ChevronRight />
          </Button>
        </div>
      </div>

      {children}
    </div>
  );
}

export interface TimePeriodUsageCardProps {
  usage: number;
  totalUsage: number;
  onChanged: (arg: { start: DateTime; end: DateTime }) => void;
  children: React.ReactNode;
  isLoading: boolean;
}

export function DayUsageCard({
  usage,
  totalUsage,
  onChanged,
  children,
  isLoading,
}: TimePeriodUsageCardProps) {
  const today = useToday();

  const props = useMemo(
    () => ({
      start: today.startOf("day"),
      endFn(start: DateTime) {
        return start.plus({ day: 1 });
      },
      nextFn(start: DateTime) {
        return start.plus({ day: 1 });
      },
      prevFn(start: DateTime) {
        return start.minus({ day: 1 });
      },
      titleFn(dt: DateTime) {
        if (+today.startOf("day") === +dt) return "Today";
        if (+today.startOf("day").minus({ day: 1 }) === +dt) return "Yesterday";
        const format = dt.year === today.year ? "dd LLL" : "dd LLL yyyy";
        return dt.toFormat(format);
      },
    }),
    [today],
  );

  return (
    <UsageCard
      usage={usage}
      totalUsage={totalUsage}
      children={children}
      onChanged={onChanged}
      isLoading={isLoading}
      {...props}
    />
  );
}

export function WeekUsageCard({
  usage,
  totalUsage,
  onChanged,
  children,
  isLoading,
}: TimePeriodUsageCardProps) {
  const today = useToday();

  const props = useMemo(
    () => ({
      start: today.startOf("week"),
      endFn(start: DateTime) {
        return start.plus({ week: 1 });
      },
      nextFn(start: DateTime) {
        return start.plus({ week: 1 });
      },
      prevFn(start: DateTime) {
        return start.minus({ week: 1 });
      },
      titleFn(dt: DateTime) {
        if (+today.startOf("week") === +dt) return "This Week";
        if (+today.startOf("week").minus({ week: 1 }) === +dt)
          return "Last Week";
        const format =
          dt.year === today.year && dt.endOf("week").year === today.year
            ? "dd LLL"
            : "dd LLL yyyy";
        return dt.toFormat(format) + " - " + dt.endOf("week").toFormat(format);
      },
    }),
    [today],
  );

  return (
    <UsageCard
      usage={usage}
      totalUsage={totalUsage}
      children={children}
      onChanged={onChanged}
      isLoading={isLoading}
      {...props}
    />
  );
}

export function MonthUsageCard({
  usage,
  totalUsage,
  onChanged,
  children,
  isLoading,
}: TimePeriodUsageCardProps) {
  const today = useToday();

  const props = useMemo(
    () => ({
      start: today.startOf("month"),
      endFn(start: DateTime) {
        return start.plus({ month: 1 });
      },
      nextFn(start: DateTime) {
        return start.plus({ month: 1 });
      },
      prevFn(start: DateTime) {
        return start.minus({ month: 1 });
      },
      titleFn(dt: DateTime) {
        if (+today.startOf("month") === +dt) return "This Month";
        if (+today.startOf("month").minus({ month: 1 }) === +dt)
          return "Last Month";
        const format = dt.year === today.year ? "LLL" : "LLL yyyy";
        return dt.toFormat(format);
      },
    }),
    [today],
  );

  return (
    <UsageCard
      usage={usage}
      totalUsage={totalUsage}
      children={children}
      onChanged={onChanged}
      isLoading={isLoading}
      {...props}
    />
  );
}

export function YearUsageCard({
  usage,
  totalUsage,
  onChanged,
  children,
  isLoading,
}: TimePeriodUsageCardProps) {
  const today = useToday();

  const props = useMemo(
    () => ({
      start: today.startOf("year"),
      endFn(start: DateTime) {
        return start.plus({ year: 1 });
      },
      nextFn(start: DateTime) {
        return start.plus({ year: 1 });
      },
      prevFn(start: DateTime) {
        return start.minus({ year: 1 });
      },
      titleFn(dt: DateTime) {
        if (+today.startOf("year") === +dt) return "This Year";
        if (+today.startOf("year").minus({ year: 1 }) === +dt)
          return "Last Year";
        return dt.toFormat("yyyy");
      },
    }),
    [today],
  );

  return (
    <UsageCard
      usage={usage}
      totalUsage={totalUsage}
      children={children}
      onChanged={onChanged}
      isLoading={isLoading}
      {...props}
    />
  );
}

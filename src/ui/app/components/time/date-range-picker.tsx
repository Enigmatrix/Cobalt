import { useToday } from "@/hooks/use-time";
import { type Interval, toHumanDateTime } from "@/lib/time";
import { cn } from "@/lib/utils";
import {
  Popover,
  PopoverTrigger,
  PopoverContent,
} from "@/components/ui/popover";
import type { ClassValue } from "clsx";
import { DateTime } from "luxon";
import { useCallback, useMemo, useState, type ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Calendar } from "@/components/ui/calendar";
import type { DateRange } from "react-day-picker";

type DateRangePickerProps = {
  value: Interval | null;
  onChange: (value: Interval | null) => void;
  render?: ReactNode;
  dayGranularity?: boolean;
  className?: ClassValue;
};

type QuickRange = {
  label: string;
  start: DateTime;
  end: DateTime;
};

function generateQuickRanges(today: DateTime): QuickRange[] {
  return [
    {
      label: "Today",
      start: today.startOf("day"),
      end: today.plus({ day: 1 }).startOf("day"),
    },
    {
      label: "Yesterday",
      start: today.minus({ day: 1 }).startOf("day"),
      end: today.startOf("day"),
    },
    {
      label: "This Week",
      start: today.startOf("week"),
      end: today.plus({ week: 1 }).startOf("week"),
    },
    {
      label: "Last Week",
      start: today.minus({ week: 1 }).startOf("week"),
      end: today.startOf("week"),
    },
    {
      label: "This Month",
      start: today.startOf("month"),
      end: today.plus({ month: 1 }).startOf("month"),
    },
    {
      label: "Last Month",
      start: today.minus({ month: 1 }).startOf("month"),
      end: today
        .minus({ month: 1 })
        .startOf("month")
        .plus({ month: 1 })
        .startOf("month"),
    },
    {
      label: "This Year",
      start: today.startOf("year"),
      end: today.plus({ year: 1 }).startOf("year"),
    },
    {
      label: "Last Year",
      start: today.minus({ year: 1 }).startOf("year"),
      end: today
        .minus({ year: 1 })
        .startOf("year")
        .plus({ year: 1 })
        .startOf("year"),
    },
  ];
}

export function DateRangePicker({
  value,
  onChange,
  render,
  dayGranularity = false,
  className,
}: DateRangePickerProps) {
  const today = useToday();
  const quickRanges = useMemo(() => generateQuickRanges(today), [today]);

  const [open, setOpen] = useState(false);

  // value but can be partially-filled
  const [inner, setInnerInner] = useState<Partial<Interval> | null>(value);

  const setInner = useCallback(
    (partial: Partial<Interval> | null) => {
      setInnerInner(partial);
      if (partial?.start && partial?.end) {
        onChange({ start: partial.start, end: partial.end });
      } else {
        onChange(null);
      }
    },
    [onChange, setInnerInner],
  );

  const calendarValue = useMemo(
    () =>
      inner
        ? { from: inner.start?.toJSDate(), to: inner.end?.toJSDate() }
        : undefined,
    [inner],
  );
  const setCalendarValue = useCallback(
    (dateRange?: DateRange) => {
      setInner(
        !dateRange
          ? null
          : {
              start: dateRange.from
                ? DateTime.fromJSDate(dateRange.from)
                : undefined,
              end: dateRange.to ? DateTime.fromJSDate(dateRange.to) : undefined,
            },
      );
    },
    [setInner],
  );

  const startStr = useMemo(
    () => (inner?.start ? htmlFormat(inner.start, dayGranularity) : ""),
    [inner?.start, dayGranularity],
  );
  const endStr = useMemo(
    () => (inner?.end ? htmlFormat(inner.end, dayGranularity) : ""),
    [inner?.end, dayGranularity],
  );

  const setStartStr = useCallback(
    (str: string) => {
      if (validHtmlFormat(str, dayGranularity)) {
        const newValue = { start: DateTime.fromISO(str), end: inner?.end };
        setInner(newValue);
      }
    },
    [setInner, inner?.end, dayGranularity],
  );

  const setEndStr = useCallback(
    (str: string) => {
      if (validHtmlFormat(str, dayGranularity)) {
        const newValue = { start: inner?.start, end: DateTime.fromISO(str) };
        setInner(newValue);
      }
    },
    [setInner, inner?.start, dayGranularity],
  );

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        {render || (
          <Button
            id="date"
            variant={"outline"}
            className={cn(
              "min-w-[300px] justify-start text-left font-normal",
              !value && "text-muted-foreground",
              className,
            )}
          >
            {formatDateRange(value, quickRanges)}
          </Button>
        )}
      </PopoverTrigger>
      <PopoverContent className="w-auto p-0 flex" align="start">
        <div className="flex flex-col">
          <div className="flex">
            <div className="flex-1 p-4">
              <Label>From</Label>
              <Input
                type={dayGranularity ? "date" : "datetime-local"}
                step={dayGranularity ? undefined : "1"}
                className="mt-2 [&:not(dark)]:[color-scheme:light] dark:[color-scheme:dark]"
                value={startStr}
                onChange={(e) => setStartStr(e.target.value)}
              />
            </div>
            <div className="flex-1 p-4">
              <Label>To</Label>
              <Input
                type={dayGranularity ? "date" : "datetime-local"}
                step={dayGranularity ? undefined : "1"}
                className="mt-2 [&:not(dark)]:[color-scheme:light] dark:[color-scheme:dark]"
                value={endStr}
                onChange={(e) => setEndStr(e.target.value)}
              />
            </div>
          </div>
          <Calendar
            initialFocus
            mode="range"
            defaultMonth={value?.start.toJSDate()}
            selected={calendarValue}
            onSelect={setCalendarValue}
            numberOfMonths={2}
          />
        </div>
        <div className="flex flex-col p-2 gap-2">
          {quickRanges.map((range) => (
            <Button
              key={range.label}
              variant="outline"
              size="sm"
              className="w-full justify-end min-w-32"
              onClick={() => {
                setInner({
                  start: range.start,
                  end: range.end,
                });
                setOpen(false);
              }}
            >
              {range.label}
            </Button>
          ))}
          <Button
            variant="destructive"
            size="sm"
            onClick={() => {
              setInner(null);
              setOpen(false);
            }}
          >
            Clear
          </Button>
        </div>
      </PopoverContent>
    </Popover>
  );
}

const formatDateRange = (date: Interval | null, ranges: QuickRange[]) => {
  if (date?.start !== undefined && date.end !== undefined) {
    const range = ranges.find(
      (r) => +r.start === +(date.start ?? 0) && +r.end === +(date.end ?? 0),
    );
    if (range) {
      return range.label;
    }
    return (
      <>
        {toHumanDateTime(date.start)} - {toHumanDateTime(date.end)}
      </>
    );
  }
  return <span>Pick a time range</span>;
};

const htmlFormat = (date: DateTime, dayGranularity: boolean) => {
  return dayGranularity
    ? date.toFormat("yyyy-MM-dd")
    : date.toFormat("yyyy-MM-dd'T'HH:mm:ss");
};

const validHtmlFormat = (str: string, dayGranularity: boolean) =>
  dayGranularity
    ? DateTime.fromFormat(str, "yyyy-MM-dd").isValid
    : DateTime.fromFormat(str, "yyyy-MM-dd'T'HH:mm").isValid ||
      DateTime.fromFormat(str, "yyyy-MM-dd'T'HH:mm:ss").isValid;

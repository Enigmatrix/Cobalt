import * as React from "react";
import { Calendar as CalendarIcon } from "lucide-react";
import type { DateRange as RDateRange } from "react-day-picker";

import { cn } from "@/lib/utils";
import { Button, type ButtonProps } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { DateTime } from "luxon";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toHumanDateTime } from "@/lib/time";
import { useToday } from "@/hooks/use-today";

type DateTimeRangePickerProps = {
  className?: string;
  date?: DateRange;
  setDate: React.Dispatch<React.SetStateAction<DateRange | undefined>>;
  showIcon?: boolean;
  formatDateRange?: (
    date: DateRange | undefined,
    ranges: QuickRange[],
  ) => React.ReactNode;
} & ButtonProps;

export type DateRange = RDateRange;

export function useDateTimeRange() {
  const [date, setDate] = React.useState<DateRange | undefined>(undefined);
  const range = React.useMemo(() => {
    if (date?.from !== undefined && date.to !== undefined) {
      return {
        from: DateTime.fromJSDate(date.from),
        to: DateTime.fromJSDate(date.to),
      };
    }
  }, [date]);
  return { range, date, setDate };
}

const htmlFormat = (date: Date) => {
  return DateTime.fromJSDate(date).toFormat("yyyy-MM-dd'T'HH:mm:ss");
};
const validDateFormat = (str: string) =>
  DateTime.fromFormat(str, "yyyy-MM-dd'T'HH:mm").isValid ||
  DateTime.fromFormat(str, "yyyy-MM-dd'T'HH:mm:ss").isValid;

const formatDateRange = (date: DateRange | undefined, ranges: QuickRange[]) => {
  if (date?.from !== undefined && date.to !== undefined) {
    const range = ranges.find(
      (r) => +r.from === +(date.from ?? 0) && +r.to === +(date.to ?? 0),
    );
    if (range) {
      return range.label;
    }
    return (
      <>
        {formatHuman(date.from)} - {formatHuman(date.to)}
      </>
    );
  }
  return date?.from ? (
    <>
      {formatHuman(date.from)} -{" "}
      <div className="text-muted-foreground">Pick end time</div>
    </>
  ) : (
    <span>Pick a time range</span>
  );
};

const formatHuman = (date: Date) => {
  const dt = DateTime.fromJSDate(date);
  return toHumanDateTime(dt);
};

type QuickRange = {
  label: string;
  from: Date;
  to: Date;
};

function generateRanges(today: DateTime): QuickRange[] {
  return [
    {
      label: "Today",
      from: today.startOf("day").toJSDate(),
      to: today.plus({ day: 1 }).startOf("day").toJSDate(),
    },
    {
      label: "Yesterday",
      from: today.minus({ day: 1 }).startOf("day").toJSDate(),
      to: today.startOf("day").toJSDate(),
    },
    {
      label: "This Week",
      from: today.startOf("week").toJSDate(),
      to: today.plus({ week: 1 }).startOf("week").toJSDate(),
    },
    {
      label: "Last Week",
      from: today.minus({ week: 1 }).startOf("week").toJSDate(),
      to: today.startOf("week").toJSDate(),
    },
    {
      label: "This Month",
      from: today.startOf("month").toJSDate(),
      to: today.plus({ month: 1 }).startOf("month").toJSDate(),
    },
    {
      label: "Last Month",
      from: today.minus({ month: 1 }).startOf("month").toJSDate(),
      to: today.startOf("month").toJSDate(),
    },
    {
      label: "This Year",
      from: today.startOf("year").toJSDate(),
      to: today.plus({ year: 1 }).startOf("year").toJSDate(),
    },
    {
      label: "Last Year",
      from: today.minus({ year: 1 }).startOf("year").toJSDate(),
      to: today.startOf("year").toJSDate(),
    },
  ];
}

export function DateTimeRangePicker({
  className,
  date,
  setDate: setDateInner,
  formatDateRange: formatDateRangeInner,
  showIcon = true,
  ...props
}: DateTimeRangePickerProps) {
  const today = useToday();

  const [open, setOpen] = React.useState(false);

  const [fromStr, setFromStrInner] = React.useState("");
  const [toStr, setToStrInner] = React.useState("");

  const setDate: React.Dispatch<React.SetStateAction<DateRange | undefined>> =
    React.useCallback(
      (
        dateArg:
          | DateRange
          | undefined
          | ((f: DateRange | undefined) => DateRange | undefined),
      ) => {
        if (typeof dateArg === "function") {
          setDateInner((date) => {
            const newDate = dateArg(date);
            setFromStrInner(newDate?.from ? htmlFormat(newDate.from) : "");
            setToStrInner(newDate?.to ? htmlFormat(newDate.to) : "");
            return newDate;
          });
        } else {
          setFromStrInner(dateArg?.from ? htmlFormat(dateArg.from) : "");
          setToStrInner(dateArg?.to ? htmlFormat(dateArg.to) : "");
          setDateInner(dateArg);
        }
      },
      [setDateInner],
    );

  const setFromStr = React.useCallback(
    (fromStr: string) => {
      setFromStrInner(fromStr);
      if (validDateFormat(fromStr)) {
        setDateInner((date) => ({
          from: new Date(fromStr),
          to: date?.to,
        }));
      }
    },
    [setDateInner],
  );

  const setToStr = React.useCallback(
    (toStr: string) => {
      setToStrInner(toStr);
      if (validDateFormat(toStr)) {
        setDateInner((date) => ({
          to: new Date(toStr),
          from: date?.from,
        }));
      }
    },
    [setDateInner],
  );
  const quickRanges = React.useMemo(() => generateRanges(today), [today]);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          id="date"
          variant={"outline"}
          className={cn(
            "min-w-[300px] justify-start text-left font-normal",
            !date && "text-muted-foreground",
            className,
          )}
          {...props}
        >
          {showIcon && <CalendarIcon />}
          {(formatDateRangeInner ?? formatDateRange)(date, quickRanges)}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-auto p-0 flex" align="start">
        <div className="flex flex-col">
          <div className="flex">
            <div className="flex-1 p-4">
              <Label>From</Label>
              <Input
                type="datetime-local"
                step="1"
                className="mt-2 [&:not(dark)]:[color-scheme:light] dark:[color-scheme:dark]"
                value={fromStr}
                onChange={(e) => setFromStr(e.target.value)}
              />
            </div>
            <div className="flex-1 p-4">
              <Label>To</Label>
              <Input
                type="datetime-local"
                step="1"
                className="mt-2 [&:not(dark)]:[color-scheme:light] dark:[color-scheme:dark]"
                value={toStr}
                onChange={(e) => setToStr(e.target.value)}
              />
            </div>
          </div>
          <Calendar
            initialFocus
            mode="range"
            defaultMonth={date?.from}
            selected={date}
            onSelect={setDate}
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
                setDate({
                  from: range.from,
                  to: range.to,
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
              setDate(undefined);
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

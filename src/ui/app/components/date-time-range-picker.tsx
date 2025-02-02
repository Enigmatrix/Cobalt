import * as React from "react";
import { Calendar as CalendarIcon } from "lucide-react";
import type { DateRange as RDateRange } from "react-day-picker";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Separator } from "@/components/ui/separator";
import { DateTime } from "luxon";
import { Input } from "@/components/ui/input";
import { Label } from "./ui/label";
type DatePickerWithRangeProps = {
  className?: string;
  date?: DateRange;
  setDate: React.Dispatch<React.SetStateAction<DateRange | undefined>>;
} & React.HTMLAttributes<HTMLDivElement>;

export type DateRange = RDateRange;

const htmlFormat = (date: Date) => {
  return DateTime.fromJSDate(date).toFormat("yyyy-MM-dd'T'HH:mm:ss");
};
const validDateFormat = (str: string) =>
  DateTime.fromFormat(str, "yyyy-MM-dd'T'HH:mm").isValid ||
  DateTime.fromFormat(str, "yyyy-MM-dd'T'HH:mm:ss").isValid;

const formatHuman = (date: Date) => {
  const dt = DateTime.fromJSDate(date);
  // if time part is 00:00:00, then return date part only
  if (dt.toFormat("HH:mm:ss") === "00:00:00") {
    return dt.toFormat("LLL dd, y");
  }
  if (dt.toFormat("ss") === "00") {
    return dt.toFormat("LLL dd, y hh:mm a");
  }
  return dt.toFormat("LLL dd, y hh:mm:ss a");
};

export function DatePickerWithRange({
  className,
  date,
  setDate: setDateInner,
  ...props
}: DatePickerWithRangeProps) {
  const now = DateTime.now();

  const [fromStr, setFromStrInner] = React.useState("");
  const [toStr, setToStrInner] = React.useState("");

  const setDate: React.Dispatch<React.SetStateAction<DateRange | undefined>> =
    React.useCallback(
      (
        dateArg:
          | DateRange
          | undefined
          | ((f: DateRange | undefined) => DateRange | undefined)
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
      [setDateInner]
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
    [setDateInner]
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
    [setDateInner]
  );
  return (
    <div className={cn("grid gap-2", className)} {...props}>
      <Popover>
        <PopoverTrigger asChild>
          <Button
            id="date"
            variant={"outline"}
            className={cn(
              "min-w-[300px] justify-start text-left font-normal",
              !date && "text-muted-foreground"
            )}
          >
            <CalendarIcon />
            {date?.from ? (
              date.to ? (
                <>
                  {formatHuman(date.from)} - {formatHuman(date.to)}
                </>
              ) : (
                <>
                  {formatHuman(date.from)} -{" "}
                  <div className="text-muted-foreground">Pick end time</div>
                </>
              )
            ) : (
              <span>Pick a time range</span>
            )}
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
                  className="mt-2"
                  value={fromStr}
                  onChange={(e) => setFromStr(e.target.value)}
                />
              </div>
              <div className="flex-1 p-4">
                <Label>To</Label>
                <Input
                  type="datetime-local"
                  step="1"
                  className="mt-2"
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
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() =>
                setDate({
                  from: now.startOf("day").toJSDate(),
                  to: now.endOf("day").toJSDate(),
                })
              }
            >
              Today
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() =>
                setDate({
                  from: now.minus({ day: 1 }).startOf("day").toJSDate(),
                  to: now.minus({ day: 1 }).endOf("day").toJSDate(),
                })
              }
            >
              Yesterday
            </Button>
            <Separator />
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() =>
                setDate({
                  from: now.startOf("week").toJSDate(),
                  to: now.endOf("week").toJSDate(),
                })
              }
            >
              This Week
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() =>
                setDate({
                  from: now.minus({ week: 1 }).startOf("week").toJSDate(),
                  to: now.minus({ week: 1 }).endOf("week").toJSDate(),
                })
              }
            >
              Last Week
            </Button>
            <Separator />
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() =>
                setDate({
                  from: now.startOf("month").toJSDate(),
                  to: now.endOf("month").toJSDate(),
                })
              }
            >
              This Month
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() =>
                setDate({
                  from: now.minus({ month: 1 }).startOf("month").toJSDate(),
                  to: now.minus({ month: 1 }).endOf("month").toJSDate(),
                })
              }
            >
              Last Month
            </Button>
            <Separator />
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() =>
                setDate({
                  from: now.startOf("year").toJSDate(),
                  to: now.endOf("year").toJSDate(),
                })
              }
            >
              This Year
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() =>
                setDate({
                  from: now.minus({ year: 1 }).startOf("year").toJSDate(),
                  to: now.minus({ year: 1 }).endOf("year").toJSDate(),
                })
              }
            >
              Last Year
            </Button>
          </div>
        </PopoverContent>
      </Popover>
    </div>
  );
}

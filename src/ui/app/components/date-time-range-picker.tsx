import * as React from "react";
import { format } from "date-fns";
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
type DatePickerWithRangeProps = {
  className?: string;
  date?: DateRange;
  setDate: React.Dispatch<React.SetStateAction<DateRange | undefined>>;
} & React.HTMLAttributes<HTMLDivElement>;

export type DateRange = RDateRange;

export function DatePickerWithRange({
  className,
  date,
  setDate,
  ...props
}: DatePickerWithRangeProps) {
  const now = DateTime.now();
  return (
    <div className={cn("grid gap-2", className)} {...props}>
      <Popover>
        <PopoverTrigger asChild>
          <Button
            id="date"
            variant={"outline"}
            className={cn(
              "w-[300px] justify-start text-left font-normal",
              !date && "text-muted-foreground"
            )}
          >
            <CalendarIcon />
            {date?.from ? (
              date.to ? (
                <>
                  {format(date.from, "LLL dd, y")} -{" "}
                  {format(date.to, "LLL dd, y")}
                </>
              ) : (
                format(date.from, "LLL dd, y")
              )
            ) : (
              <span>Pick a time range</span>
            )}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0 flex" align="start">
          <div>
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

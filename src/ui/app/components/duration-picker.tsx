import * as React from "react";
import { Hourglass } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button, type ButtonProps } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Duration } from "luxon";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import parse from "parse-duration";
import { Separator } from "@/components/ui/separator";
import { toHumanDurationFull } from "@/lib/time";

// disable parsing of these units
["y", "mo", "ms", "us", "ns"].forEach((u) => (parse.unit[u] = 0));

type DurationPickerProps = {
  className?: string;
  duration?: Duration;
  setDuration: React.Dispatch<React.SetStateAction<Duration | undefined>>;
} & ButtonProps;

const formatHuman = (duration: Duration) => {
  return toHumanDurationFull(duration);
};

function daysStr(duration?: Duration) {
  const days = duration?.as("days") ?? 0;
  return Math.floor(days).toString();
}

export function DurationPicker({
  className,
  duration,
  setDuration,
  ...props
}: DurationPickerProps) {
  const [open, setOpen] = React.useState(false);
  const [quickInput, setQuickInputInner] = React.useState("");
  function setQuickInput(value: string) {
    setQuickInputInner(value);
    const parsed = parse(value);
    if (parsed) {
      const raw = Duration.fromMillis(parsed);
      const dt = Duration.fromObject({
        days: Math.floor(raw.as("days")),
        hours: raw.rescale().hours,
        minutes: raw.rescale().minutes,
        seconds: raw.rescale().seconds,
        milliseconds: raw.rescale().milliseconds,
      });
      setDuration(dt);
      setSecondsInner(dt.seconds.toString());
      setMinutesInner(dt.minutes.toString());
      setHoursInner(dt.hours.toString());
      setDaysInner(daysStr(dt));
    }
  }

  const [days, setDaysInner] = React.useState(daysStr(duration));
  const [hours, setHoursInner] = React.useState(
    (duration?.rescale()?.hours || 0).toString()
  );
  const [minutes, setMinutesInner] = React.useState(
    (duration?.rescale()?.minutes || 0).toString()
  );
  const [seconds, setSecondsInner] = React.useState(
    (duration?.rescale()?.seconds || 0).toString()
  );

  const setDays = (value: string) => {
    setDaysInner(value);
    handleUnitChange("days", value);
    setQuickInput("");
  };
  const setHours = (value: string) => {
    setHoursInner(value);
    handleUnitChange("hours", value);
    setQuickInput("");
  };
  const setMinutes = (value: string) => {
    setMinutesInner(value);
    handleUnitChange("minutes", value);
    setQuickInput("");
  };
  const setSeconds = (value: string) => {
    setSecondsInner(value);
    handleUnitChange("seconds", value);
    setQuickInput("");
  };
  function resetText(e: React.ChangeEvent<HTMLInputElement>) {
    if (e.target.value === "") {
      e.target.value = "0";
    }
  }

  function handleUnitChange(unit: keyof Duration, value: string) {
    setDuration(
      (duration ?? Duration.fromMillis(0))
        .set({
          [unit]: value === "" ? 0 : Number.parseInt(value),
        })
        .rescale()
    );
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          id="date"
          variant={"outline"}
          className={cn(
            "min-w-[300px] justify-start text-left font-normal",
            !duration && "text-muted-foreground",
            className
          )}
          {...props}
        >
          <Hourglass />
          {duration ? (
            <>{formatHuman(duration)}</>
          ) : (
            <span>Pick a duration</span>
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-auto p-4 flex flex-col gap-4" align="start">
        <div>
          <Label className="font-semibold">Quick input</Label>
          <Input
            value={quickInput}
            onChange={(e) => setQuickInput(e.target.value)}
            placeholder="e.g. 2d 4h 30m"
            className="mt-2"
          />
        </div>
        <Separator orientation="horizontal" />
        <div className="grid gap-2">
          <div className="grid grid-cols-3 items-center gap-4">
            <Label>Days</Label>
            <Input
              type="number"
              value={days}
              onChange={(e) => setDays(e.target.value)}
              min={0}
              className="col-span-2"
              onBlur={resetText}
            />
          </div>
          <div className="grid grid-cols-3 items-center gap-4">
            <Label>Hours</Label>
            <Input
              type="number"
              value={hours}
              onChange={(e) => setHours(e.target.value)}
              min={0}
              max={23}
              className="col-span-2"
              onBlur={resetText}
            />
          </div>
          <div className="grid grid-cols-3 items-center gap-4">
            <Label>Minutes</Label>
            <Input
              type="number"
              value={minutes}
              onChange={(e) => setMinutes(e.target.value)}
              min={0}
              max={59}
              className="col-span-2"
              onBlur={resetText}
            />
          </div>
          <div className="grid grid-cols-3 items-center gap-4">
            <Label>Seconds</Label>
            <Input
              type="number"
              value={seconds}
              onChange={(e) => setSeconds(e.target.value)}
              min={0}
              max={59}
              className="col-span-2"
              onBlur={resetText}
            />
          </div>
          <div className="flex justify-end gap-2">
            <Button
              variant="destructive"
              onClick={() => {
                setDuration(undefined);
                setQuickInput("");
                setDaysInner("0");
                setHoursInner("0");
                setMinutesInner("0");
                setSecondsInner("0");
                setOpen(false);
              }}
            >
              Clear
            </Button>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}

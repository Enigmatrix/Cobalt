import * as React from "react";
import { Hourglass } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Duration } from "luxon";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { durationToTicks, toHumanDurationFull } from "@/lib/time";
import type { ClassValue } from "clsx";
import { Separator } from "@/components/ui/separator";

type DurationPickerProps = {
  className?: ClassValue;
  value: Duration | null;
  onValueChange: (dur: Duration | null) => void;
  showIcon?: boolean;
};

const QUICK_DURATIONS = [
  { label: "5 mins", duration: Duration.fromObject({ minutes: 5 }) },
  { label: "30 mins", duration: Duration.fromObject({ minutes: 30 }) },
  { label: "1 hr", duration: Duration.fromObject({ hours: 1 }) },
  { label: "2 hrs", duration: Duration.fromObject({ hours: 2 }) },
  { label: "1 day", duration: Duration.fromObject({ days: 1 }) },
  { label: "7 days", duration: Duration.fromObject({ days: 7 }) },
];

export function DurationPicker({
  className,
  value: duration,
  onValueChange: setDuration,
  showIcon = true,
}: DurationPickerProps) {
  const [open, setOpen] = React.useState(false);
  const [days, setDays] = React.useState(
    Math.floor(duration?.as("days") ?? 0).toString(),
  );
  const [hours, setHours] = React.useState(
    Math.floor((duration?.as("hours") ?? 0) % 24).toString(),
  );
  const [minutes, setMinutes] = React.useState(
    Math.floor((duration?.as("minutes") ?? 0) % 60).toString(),
  );
  const [seconds, setSeconds] = React.useState(
    Math.floor((duration?.as("seconds") ?? 0) % 60).toString(),
  );

  const handleDurationChange = (
    days: string,
    hours: string,
    minutes: string,
    seconds: string,
  ) => {
    const d = parseInt(days) || 0;
    const h = parseInt(hours) || 0;
    const m = parseInt(minutes) || 0;
    const s = parseInt(seconds) || 0;

    if (d === 0 && h === 0 && m === 0 && s === 0) {
      setDuration(null);
    } else {
      setDuration(
        Duration.fromObject({
          days: d,
          hours: h,
          minutes: m,
          seconds: s,
        }),
      );
    }
  };

  const handleQuickDuration = (quickDuration: Duration) => {
    setDuration(quickDuration);
    setDays(Math.floor(quickDuration.as("days")).toString());
    setHours(Math.floor(quickDuration.as("hours") % 24).toString());
    setMinutes(Math.floor(quickDuration.as("minutes") % 60).toString());
    setSeconds(Math.floor(quickDuration.as("seconds") % 60).toString());
    setOpen(false);
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          className={cn(
            "w-[280px] justify-start text-left font-normal",
            !duration && "text-muted-foreground",
            className,
          )}
        >
          {showIcon && <Hourglass className="mr-2 h-4 w-4" />}
          {duration
            ? toHumanDurationFull(durationToTicks(duration))
            : "Pick duration"}
        </Button>
      </PopoverTrigger>
      <PopoverContent
        className="min-w-[300px] w-[--radix-popover-trigger-width] p-4"
        align="start"
      >
        <div className="space-y-4">
          <div className="grid grid-cols-3 gap-2">
            {QUICK_DURATIONS.map((quick) => (
              <Button
                key={quick.label}
                variant="outline"
                size="sm"
                onClick={() => handleQuickDuration(quick.duration)}
                className="h-8"
              >
                {quick.label}
              </Button>
            ))}
          </div>

          <Separator />

          <div className="grid grid-cols-4 gap-3">
            <div className="space-y-1">
              <Label className="text-xs">Days</Label>
              <Input
                type="number"
                value={days}
                onChange={(e) => {
                  setDays(e.target.value);
                  handleDurationChange(e.target.value, hours, minutes, seconds);
                }}
                min={0}
                className="h-8"
              />
            </div>
            <div className="space-y-1">
              <Label className="text-xs">Hours</Label>
              <Input
                type="number"
                value={hours}
                onChange={(e) => {
                  setHours(e.target.value);
                  handleDurationChange(days, e.target.value, minutes, seconds);
                }}
                min={0}
                max={23}
                className="h-8"
              />
            </div>
            <div className="space-y-1">
              <Label className="text-xs">Mins</Label>
              <Input
                type="number"
                value={minutes}
                onChange={(e) => {
                  setMinutes(e.target.value);
                  handleDurationChange(days, hours, e.target.value, seconds);
                }}
                min={0}
                max={59}
                className="h-8"
              />
            </div>
            <div className="space-y-1">
              <Label className="text-xs">Secs</Label>
              <Input
                type="number"
                value={seconds}
                onChange={(e) => {
                  setSeconds(e.target.value);
                  handleDurationChange(days, hours, minutes, e.target.value);
                }}
                min={0}
                max={59}
                className="h-8"
              />
            </div>
          </div>

          <Button
            variant="destructive"
            size="sm"
            className="w-full h-8"
            onClick={() => {
              setDuration(null);
              setDays("0");
              setHours("0");
              setMinutes("0");
              setSeconds("0");
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

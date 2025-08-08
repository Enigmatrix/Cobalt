import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Separator } from "@/components/ui/separator";
import { durationToTicks, toHumanDurationFull } from "@/lib/time";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { Hourglass } from "lucide-react";
import { Duration } from "luxon";
import * as React from "react";

interface DurationPickerProps {
  className?: ClassValue;
  value: Duration | null;
  onValueChange: (dur: Duration | null) => void;
  showIcon?: boolean;
  showClear?: boolean;
}

const QUICK_DURATIONS = [
  { label: "5 mins", duration: Duration.fromObject({ minutes: 5 }) },
  { label: "30 mins", duration: Duration.fromObject({ minutes: 30 }) },
  { label: "1 hr", duration: Duration.fromObject({ hours: 1 }) },
  { label: "2 hrs", duration: Duration.fromObject({ hours: 2 }) },
  { label: "1 day", duration: Duration.fromObject({ days: 1 }) },
  { label: "7 days", duration: Duration.fromObject({ days: 7 }) },
];

// Helper function to break down duration into components
const breakdownDuration = (duration: Duration | null) => {
  if (!duration) {
    return { days: 0, hours: 0, minutes: 0, seconds: 0 };
  }

  return {
    days: Math.floor(duration.as("days")),
    hours: Math.floor(duration.as("hours") % 24),
    minutes: Math.floor(duration.as("minutes") % 60),
    seconds: Math.floor(duration.as("seconds") % 60),
  };
};

export function DurationPicker({
  className,
  value: duration,
  onValueChange: setDuration,
  showIcon = true,
  showClear = true,
}: DurationPickerProps) {
  const [open, setOpen] = React.useState(false);

  // Derive the breakdown from the current duration
  const { days, hours, minutes, seconds } = breakdownDuration(duration);

  const handleInputChange = (
    field: "days" | "hours" | "minutes" | "seconds",
    value: string,
  ) => {
    const numValue = parseInt(value) || 0;

    // Create new duration with updated field
    const newValues = { days, hours, minutes, seconds, [field]: numValue };
    const newDuration = Duration.fromObject(newValues);

    setDuration(newDuration);
  };

  const handleQuickDuration = (quickDuration: Duration) => {
    setDuration(quickDuration);
    setOpen(false);
  };

  const handleClear = () => {
    setDuration(null);
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
        className="min-w-[300px] w-(--radix-popover-trigger-width) p-4"
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
                value={days.toString()}
                onChange={(e) => handleInputChange("days", e.target.value)}
                min={0}
                className="h-8"
              />
            </div>
            <div className="space-y-1">
              <Label className="text-xs">Hours</Label>
              <Input
                type="number"
                value={hours.toString()}
                onChange={(e) => handleInputChange("hours", e.target.value)}
                min={0}
                max={23}
                className="h-8"
              />
            </div>
            <div className="space-y-1">
              <Label className="text-xs">Mins</Label>
              <Input
                type="number"
                value={minutes.toString()}
                onChange={(e) => handleInputChange("minutes", e.target.value)}
                min={0}
                max={59}
                className="h-8"
              />
            </div>
            <div className="space-y-1">
              <Label className="text-xs">Secs</Label>
              <Input
                type="number"
                value={seconds.toString()}
                onChange={(e) => handleInputChange("seconds", e.target.value)}
                min={0}
                max={59}
                className="h-8"
              />
            </div>
          </div>

          {showClear && (
            <Button
              variant="destructive"
              size="sm"
              className="w-full h-8"
              onClick={handleClear}
            >
              Clear
            </Button>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}

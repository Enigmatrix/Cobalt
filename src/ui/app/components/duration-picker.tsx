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

type DurationPickerProps = {
  className?: string;
  duration?: Duration;
  setDuration: React.Dispatch<React.SetStateAction<Duration | undefined>>;
  disabled?: boolean;
} & React.HTMLAttributes<HTMLDivElement>;

const formatHuman = (duration: Duration) => {
  return "TODO";
};

export function DurationPicker({
  className,
  duration,
  setDuration,
  disabled,
  ...props
}: DurationPickerProps) {
  return (
    <div className={cn("grid gap-2", className)} {...props}>
      <Popover>
        <PopoverTrigger asChild disabled={disabled}>
          <Button
            id="date"
            variant={"outline"}
            className={cn(
              "min-w-[300px] justify-start text-left font-normal",
              !duration && "text-muted-foreground"
            )}
          >
            <Hourglass />
            {duration ? (
              <>{formatHuman(duration)}</>
            ) : (
              <span>Pick a duration</span>
            )}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0 flex" align="start">
          TODO
        </PopoverContent>
      </Popover>
    </div>
  );
}

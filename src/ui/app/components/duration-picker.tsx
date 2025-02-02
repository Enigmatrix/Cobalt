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

type DurationPickerProps = {
  className?: string;
  duration?: Duration;
  setDuration: React.Dispatch<React.SetStateAction<Duration | undefined>>;
} & ButtonProps;

const formatHuman = (duration: Duration) => {
  return "TODO";
};

export function DurationPicker({
  className,
  duration,
  setDuration,
  ...props
}: DurationPickerProps) {
  return (
    <Popover>
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
      <PopoverContent className="w-auto p-0 flex" align="start">
        TODO
      </PopoverContent>
    </Popover>
  );
}

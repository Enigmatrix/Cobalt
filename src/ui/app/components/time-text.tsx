import type { DateTime } from "luxon";
import {
  ticksToDateTime,
  toHumanDateTime,
  toHumanDateTimeFull,
} from "@/lib/time";
import { cn } from "@/lib/utils";
import {
  TooltipProvider,
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import type { ClassValue } from "clsx";

export function DateTimeText({
  ticks,
  datetime,
  className,
}: {
  ticks?: number;
  datetime?: DateTime;
  className?: ClassValue;
}) {
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <div className={cn("tracking-tighter", className)}>
            {toHumanDateTime(ticks ? ticksToDateTime(ticks) : datetime!)}
          </div>
        </TooltipTrigger>
        <TooltipContent>
          {toHumanDateTimeFull(ticks ? ticksToDateTime(ticks) : datetime!)}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

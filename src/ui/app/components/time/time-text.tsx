import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  ticksToDateTime,
  toHumanDateTime,
  toHumanDateTimeFull,
} from "@/lib/time";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import type { DateTime } from "luxon";

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

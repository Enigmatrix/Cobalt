import type { Duration } from "luxon";
import {
  durationToTicks,
  toHumanDuration,
  toHumanDurationFull,
} from "@/lib/time";
import { cn } from "@/lib/utils";
import {
  TooltipProvider,
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@/components/ui/tooltip";
import type { ClassValue } from "clsx";

export function DurationText({
  ticks,
  duration,
  className,
}: {
  ticks?: number;
  duration?: Duration;
  className?: ClassValue;
}) {
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <div
            className={cn(
              "tracking-tighter whitespace-nowrap truncate",
              className,
            )}
          >
            {toHumanDuration(ticks ?? durationToTicks(duration!))}
          </div>
        </TooltipTrigger>
        <TooltipContent>
          {toHumanDurationFull(ticks ?? durationToTicks(duration!))}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

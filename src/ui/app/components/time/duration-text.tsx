import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  durationToTicks,
  toHumanDuration,
  toHumanDurationFull,
} from "@/lib/time";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import type { Duration } from "luxon";

export function DurationText({
  ticks,
  duration,
  className,
  description,
}: {
  ticks?: number;
  duration?: Duration;
  className?: ClassValue;
  description?: string;
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
          {description ? `${description}: ` : ""}
          {toHumanDurationFull(ticks ?? durationToTicks(duration!))}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

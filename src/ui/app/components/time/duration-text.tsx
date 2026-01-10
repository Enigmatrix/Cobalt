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
  symbolForZero,
  showSymbolForZero,
}: {
  ticks?: number;
  duration?: Duration;
  className?: ClassValue;
  description?: string;
  symbolForZero?: string;
  showSymbolForZero?: boolean;
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
            {toHumanDuration(
              ticks ?? durationToTicks(duration!),
              showSymbolForZero,
              symbolForZero,
            )}
          </div>
        </TooltipTrigger>
        <TooltipContent>
          {description ? `${description}: ` : ""}
          {toHumanDurationFull(
            ticks ?? durationToTicks(duration!),
            showSymbolForZero,
            symbolForZero,
          )}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

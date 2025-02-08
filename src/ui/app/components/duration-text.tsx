import type { Duration } from "luxon";
import { durationToTicks, toHumanDuration } from "@/lib/time";
import { cn } from "@/lib/utils";

export function DurationText({
  ticks,
  duration,
  className,
}: {
  ticks?: number;
  duration?: Duration;
  className?: string;
}) {
  return (
    <div className={cn("tracking-tighter", className)}>
      {toHumanDuration(ticks ?? durationToTicks(duration!))}
    </div>
  );
}

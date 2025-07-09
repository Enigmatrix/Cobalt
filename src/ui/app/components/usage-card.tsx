import { DurationText } from "@/components/time/duration-text";
import { Button } from "@/components/ui/button";
import {
  VizCard,
  VizCardAction,
  VizCardContent,
  VizCardHeader,
  VizCardTitle,
} from "@/components/viz/viz-card";
import { useToday } from "@/hooks/use-time";
import { toHumanInterval, type Interval, type Period } from "@/lib/time";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useCallback, useMemo } from "react";

export function UsageCard({
  interval,
  onIntervalChanged,
  next,
  prev,
  canGoNext,
  canGoPrev,

  title,
  usage,
  totalUsage,
  children,
  isLoading,
}: {
  interval: Interval;
  onIntervalChanged: (interval: Interval) => void;
  next: (interval: Interval) => Interval;
  prev: (interval: Interval) => Interval;
  canGoNext: (interval: Interval) => boolean;
  canGoPrev: (interval: Interval) => boolean;
  title: (interval: Interval) => string;

  usage?: number;
  totalUsage: number;
  children: React.ReactNode;
  isLoading: boolean;
}) {
  const nextCallback = useCallback(() => {
    if (canGoNext(interval)) {
      onIntervalChanged(next(interval));
    }
  }, [interval, canGoNext, next, onIntervalChanged]);

  const prevCallback = useCallback(() => {
    if (canGoPrev(interval)) {
      onIntervalChanged(prev(interval));
    }
  }, [interval, canGoPrev, prev, onIntervalChanged]);

  return (
    <VizCard>
      <VizCardHeader className="pb-1">
        <VizCardTitle className="pl-4 pt-4">
          <div className="whitespace-nowrap text-base text-card-foreground/50">
            {title(interval)}
          </div>
          <div className="flex gap-2 items-baseline font-semibold">
            <DurationText className="text-xl" ticks={usage ?? totalUsage} />
            {usage !== undefined && usage !== 0 && (
              <>
                <span className="text-xl text-muted-foreground">/</span>
                <DurationText
                  className="text-muted-foreground"
                  ticks={totalUsage}
                />
              </>
            )}
          </div>
        </VizCardTitle>

        <VizCardAction className="flex mt-5 mr-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={prevCallback}
            disabled={!canGoPrev(interval) || isLoading}
          >
            <ChevronLeft />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={nextCallback}
            disabled={!canGoNext(interval) || isLoading}
          >
            <ChevronRight />
          </Button>
        </VizCardAction>
      </VizCardHeader>

      <VizCardContent>{children}</VizCardContent>
    </VizCard>
  );
}

export interface TimePeriodUsageCardProps {
  timePeriod: Period;
  interval: Interval;
  onIntervalChanged: (interval: Interval) => void;

  usage?: number;
  totalUsage: number;
  children: React.ReactNode;
  isLoading: boolean;
}

export function TimePeriodUsageCard({
  timePeriod,
  ...props
}: TimePeriodUsageCardProps) {
  const today = useToday();

  const iterProps = useMemo(
    () => ({
      next(interval: Interval) {
        return {
          start: interval.start.plus({ [timePeriod]: 1 }),
          end: interval.start
            .plus({ [timePeriod]: 1 })
            .plus({ [timePeriod]: 1 }),
        };
      },
      prev(interval: Interval) {
        return {
          start: interval.start.minus({ [timePeriod]: 1 }),
          end: interval.start
            .minus({ [timePeriod]: 1 })
            .plus({ [timePeriod]: 1 }),
        };
      },
      canGoNext(interval: Interval) {
        const nextStart = interval.start.plus({ [timePeriod]: 1 });
        return today.plus({ day: 1 }) > nextStart;
      },
      canGoPrev() {
        return true;
      },
      title(interval: Interval) {
        return toHumanInterval(today, interval);
      },
    }),
    [timePeriod, today],
  );

  return <UsageCard {...props} {...iterProps} />;
}

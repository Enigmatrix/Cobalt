import { DurationText } from "@/components/time/duration-text";
import { Button } from "@/components/ui/button";
import { useToday } from "@/hooks/use-time";
import type { Interval, Period } from "@/lib/time";
import { ChevronLeft, ChevronRight } from "lucide-react";
import type { DateTime } from "luxon";
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
    <div className="flex flex-col rounded-xl bg-card border border-border">
      <div className="flex items-center">
        <div className="flex flex-col p-4 pb-1 min-w-0">
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
        </div>

        <div className="flex-1" />

        {/* hide button controls between md: and lg: */}
        <div className="flex m-2 max-lg:hidden max-md:block">
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
        </div>
      </div>

      {children}
    </div>
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

const hourTitle = (today: DateTime, interval: Interval) => {
  const dt = interval.start;
  const format =
    dt.toFormat("yyyy LLL dd") === today.toFormat("yyyy LLL dd")
      ? "HH:mm"
      : "yyyy LLL dd HH:mm";
  return dt.toFormat(format);
};

const dayTitle = (today: DateTime, interval: Interval) => {
  const dt = interval.start;
  if (+today.startOf("day") === +dt) return "Today";
  if (+today.startOf("day").minus({ day: 1 }) === +dt) return "Yesterday";
  const format = dt.year === today.year ? "dd LLL" : "dd LLL yyyy";
  return dt.toFormat(format);
};

const weekTitle = (today: DateTime, interval: Interval) => {
  const dt = interval.start;
  if (+today.startOf("week") === +dt) return "This Week";
  if (+today.startOf("week").minus({ week: 1 }) === +dt) return "Last Week";
  const format =
    dt.year === today.year && dt.endOf("week").year === today.year
      ? "dd LLL"
      : "dd LLL yyyy";
  return dt.toFormat(format) + " - " + dt.endOf("week").toFormat(format);
};

const monthTitle = (today: DateTime, interval: Interval) => {
  const dt = interval.start;
  if (+today.startOf("month") === +dt) return "This Month";
  if (+today.startOf("month").minus({ month: 1 }) === +dt) return "Last Month";
  const format = dt.year === today.year ? "LLL" : "LLL yyyy";
  return dt.toFormat(format);
};

const yearTitle = (today: DateTime, interval: Interval) => {
  const dt = interval.start;
  if (+today.startOf("year") === +dt) return "This Year";
  if (+today.startOf("year").minus({ year: 1 }) === +dt) return "Last Year";
  return dt.toFormat("yyyy");
};

const titles = {
  hour: hourTitle,
  day: dayTitle,
  week: weekTitle,
  month: monthTitle,
  year: yearTitle,
};

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
        return titles[timePeriod](today, interval);
      },
    }),
    [timePeriod, today],
  );

  return <UsageCard {...props} {...iterProps} />;
}

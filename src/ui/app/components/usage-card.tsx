import { DurationText } from "@/components/time/duration-text";
import { Button } from "@/components/ui/button";
import {
  VizCard,
  VizCardAction,
  VizCardContent,
  VizCardHeader,
  VizCardTitle,
} from "@/components/viz/viz-card";
import { useToday, type IntervalControls } from "@/hooks/use-time";
import { toHumanInterval, type Interval, type Period } from "@/lib/time";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useMemo } from "react";

export interface TimePeriodUsageCardProps extends IntervalControls {
  timePeriod: Period;
  interval: Interval;

  usage?: number;
  totalUsage: number;
  children: React.ReactNode;
  isLoading: boolean;
}

export function UsageCard({
  interval,
  canGoNext,
  canGoPrev,
  goNext,
  goPrev,
  usage,
  totalUsage,
  children,
  isLoading,
}: TimePeriodUsageCardProps) {
  return (
    <VizCard>
      <VizCardHeader className="pb-1">
        <VizCardTitle className="pl-4 pt-4">
          <UsageCardTitle
            usage={usage}
            totalUsage={totalUsage}
            interval={interval}
          />
        </VizCardTitle>

        <VizCardAction className="flex mt-5 mr-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={goPrev}
            disabled={!canGoPrev || isLoading}
          >
            <ChevronLeft />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={goNext}
            disabled={!canGoNext || isLoading}
          >
            <ChevronRight />
          </Button>
        </VizCardAction>
      </VizCardHeader>

      <VizCardContent>{children}</VizCardContent>
    </VizCard>
  );
}

export function UsageCardTitle({
  usage,
  totalUsage,
  interval,
}: {
  usage?: number;
  totalUsage: number;
  interval: Interval;
}) {
  const today = useToday();
  const title = useMemo(
    () => toHumanInterval(today, interval),
    [today, interval],
  );

  return (
    <>
      <div className="whitespace-nowrap text-base text-card-foreground/50">
        {title}
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
    </>
  );
}

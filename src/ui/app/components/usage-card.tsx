import { DurationText } from "@/components/time/duration-text";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  VizCard,
  VizCardAction,
  VizCardContent,
  VizCardHeader,
  VizCardTitle,
} from "@/components/viz/viz-card";
import { useToday } from "@/hooks/use-time";
import { toHumanInterval, type Interval } from "@/lib/time";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useMemo } from "react";

export interface UsageCardProps {
  interval: Interval;

  usage?: number;
  totalUsage: number;
  children: React.ReactNode;
  actions: React.ReactNode;
}

export function UsageCard({
  interval,
  usage,
  totalUsage,
  children,
  actions,
}: UsageCardProps) {
  return (
    <VizCard>
      <VizCardHeader className="pb-4 has-data-[slot=card-action]:grid-cols-[minmax(0,1fr)_auto]">
        <VizCardTitle className="pl-4 pt-4">
          <UsageCardTitle
            usage={usage}
            totalUsage={totalUsage}
            interval={interval}
          />
        </VizCardTitle>

        <VizCardAction className="flex mt-4 mr-1.5">{actions}</VizCardAction>
      </VizCardHeader>

      <VizCardContent>{children}</VizCardContent>
    </VizCard>
  );
}

export function PrevButton({
  canGoPrev,
  isLoading,
  goPrev,
}: {
  canGoPrev: boolean;
  isLoading: boolean;
  goPrev: () => void;
}) {
  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={goPrev}
      disabled={!canGoPrev || isLoading}
    >
      <ChevronLeft />
    </Button>
  );
}

export function NextButton({
  canGoNext,
  isLoading,
  goNext,
}: {
  canGoNext: boolean;
  isLoading: boolean;
  goNext: () => void;
}) {
  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={goNext}
      disabled={!canGoNext || isLoading}
    >
      <ChevronRight />
    </Button>
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
      {/* TODO: create a interval range component? */}
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger className="max-w-full text-base text-card-foreground/50 truncate">
            {title}
          </TooltipTrigger>
          <TooltipContent>
            <div>{title}</div>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
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

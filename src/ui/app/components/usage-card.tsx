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
import { ChevronLeft, ChevronRight, Loader2 } from "lucide-react";
import { useMemo } from "react";

export interface UsageCardProps {
  interval: Interval;

  usage?: number;
  totalUsage: number;
  children: React.ReactNode;
  actions: React.ReactNode;

  isLoading?: boolean;
  isValidating?: boolean;
}

export function UsageCard({
  interval,
  isLoading,
  isValidating,
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
            isLoading={isLoading}
            isValidating={isValidating}
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
  isValidating,
  goPrev,
}: {
  canGoPrev: boolean;
  isLoading: boolean;
  isValidating: boolean;
  goPrev: () => void;
}) {
  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={goPrev}
      disabled={!canGoPrev || isLoading || isValidating}
    >
      <ChevronLeft />
    </Button>
  );
}

export function NextButton({
  canGoNext,
  isLoading,
  isValidating,
  goNext,
}: {
  canGoNext: boolean;
  isLoading: boolean;
  isValidating: boolean;
  goNext: () => void;
}) {
  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={goNext}
      disabled={!canGoNext || isLoading || isValidating}
    >
      <ChevronRight />
    </Button>
  );
}

export function UsageCardTitle({
  usage,
  totalUsage,
  interval,
  isLoading,
  isValidating,
}: {
  usage?: number;
  totalUsage: number;
  interval: Interval;
  isLoading?: boolean;
  isValidating?: boolean;
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
          <TooltipTrigger className="max-w-full text-base flex items-center gap-1.5 text-card-foreground/50 truncate">
            {title}
            {(isLoading || isValidating) && (
              <Loader2 className="size-4 animate-spin" />
            )}
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

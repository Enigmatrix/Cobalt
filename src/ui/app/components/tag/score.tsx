import { ScoreSlider } from "@/components/tag/score-slider";
import { useTheme } from "@/components/theme-provider";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Text } from "@/components/ui/text";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";

export const Colors = {
  best: "bg-green-300 dark:bg-green-800",
  neutral: "bg-gray-300 dark:bg-gray-600",
  worst: "bg-red-400 dark:bg-red-800",
};

/**
 * Converts a score value (-100 to 100) into a descriptive string
 * @param score A number between -100 and 100
 * @returns A string describing the focus level
 */
export function getScoreDescription(score: number): string {
  if (score < -80) return "Highly Distractive";
  if (score < -40) return "Distractive";
  if (score < 0) return "Slightly Distractive";
  if (score === 0) return "Neutral";
  if (score < 40) return "Slightly Focused";
  if (score < 80) return "Focused";
  return "Highly Focused";
}

export function ScoreCircle({
  score,
  className,
  hoverTooltip = true,
}: {
  score: number;
  className?: ClassValue;
  hoverTooltip?: boolean;
}) {
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger
          type="button"
          asChild={!hoverTooltip}
          className={cn(!hoverTooltip && "pointer-events-none")}
        >
          <ScoreWrapper
            score={score}
            className={cn("size-3 rounded-full", className)}
          />
        </TooltipTrigger>
        <TooltipContent>
          <Text>{`${getScoreDescription(score)} (${score})`}</Text>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

export function ScoreBadge({
  score,
  className,
  circleClassName,
}: {
  score: number;
  className?: ClassValue;
  circleClassName?: ClassValue;
}) {
  return (
    <div
      className={cn(
        "pl-2 pr-1 py-0.5 flex items-center gap-2 rounded-full border border-border bg-muted/50 hover:bg-muted text-xs",
        "ring-offset-background transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        "disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none",
        className,
      )}
    >
      <Text className="text-muted-foreground">
        {getScoreDescription(score)}
      </Text>
      <ScoreCircle
        score={score}
        className={cn("size-3", circleClassName)}
        // hoverTooltip={false}
      />
    </div>
  );
}

export function ScoreWrapper({
  score,
  className,
  children,
}: {
  score: number;
  className?: ClassValue;
  children?: React.ReactNode;
}) {
  const { theme } = useTheme();

  const getOpacity = (value: number) => {
    return Math.max(0, value / 100);
  };

  return (
    <div
      className={cn(
        "relative text-sm font-medium border-border border overflow-hidden",
        Colors.neutral,
        className,
      )}
    >
      <div
        className={cn("absolute inset-0", Colors.best)}
        style={{
          opacity: getOpacity(score),
          mixBlendMode: theme === "dark" ? "normal" : "multiply",
        }}
      />
      <div
        className={cn("absolute inset-0", Colors.worst)}
        style={{
          opacity: getOpacity(-score),
          mixBlendMode: theme === "dark" ? "normal" : "multiply",
        }}
      />
      <div className="relative">{children}</div>
    </div>
  );
}

export function ScoreEdit({
  score,
  onScoreChange,
  className,
  children,
}: {
  score: number;
  onScoreChange: (score: number) => void;
  className?: ClassValue;
  children?: React.ReactNode;
}) {
  return (
    <Popover>
      <PopoverTrigger className={cn(className)}>{children}</PopoverTrigger>
      <PopoverContent align="start" className="p-4">
        <div className="h-8 mb-3 text-sm gap-2 flex items-center">
          Score
          <div>-</div>
          <span className="text-sm text-muted-foreground min-w-0 truncate">
            {getScoreDescription(score)}
          </span>
          <span className="text-sm text-muted-foreground">({score})</span>
          {score !== 0 && (
            <Button
              variant="outline"
              size="sm"
              className="px-2 py-1 h-6 m-0 text-xs"
              onClick={() => onScoreChange(0)}
            >
              Reset
            </Button>
          )}
        </div>
        <ScoreSlider value={score} onValueChange={onScoreChange} />
      </PopoverContent>
    </Popover>
  );
}

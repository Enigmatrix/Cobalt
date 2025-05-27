import { useTheme } from "@/components/theme-provider";
import { Text } from "@/components/ui/text";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export const Colors = {
  best: "bg-green-300 dark:bg-green-800",
  neutral: "bg-gray-300 dark:bg-gray-800",
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
        <TooltipTrigger className={cn(!hoverTooltip && "pointer-events-none")}>
          <ScoreWrapper
            score={score}
            className={cn("size-4 rounded-full", className)}
          />
        </TooltipTrigger>
        <TooltipContent>
          <Text>{getScoreDescription(score)}</Text>
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
        "pl-1 pr-2 py-0.5 flex items-center gap-1 rounded-full border border-border bg-muted/50 text-xs",
        className,
      )}
    >
      <ScoreCircle
        score={score}
        className={cn("size-4", circleClassName)}
        hoverTooltip={false}
      />
      <Text className="text-muted-foreground">
        {getScoreDescription(score)}
      </Text>
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

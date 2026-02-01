import { DurationText } from "@/components/time/duration-text";
import { Text } from "@/components/ui/text";
import type { TriggerAction } from "@/lib/entities";
import { cn } from "@/lib/utils";
import { MessageSquareIcon, SunIcon, ZapIcon } from "lucide-react";

export interface TriggerActionIndicatorProps {
  action: TriggerAction;
  className?: string;
}

/**
 * Display a trigger action with icon and colored text.
 * Used in alert detail pages and cards.
 */
export function TriggerActionIndicator({
  action,
  className,
}: TriggerActionIndicatorProps) {
  switch (action.tag) {
    case "kill":
      return (
        <div
          className={cn(
            "flex items-center gap-1.5 text-muted-foreground",
            className,
          )}
        >
          <ZapIcon className="size-[0.75lh] shrink-0 text-red-600 dark:text-red-400" />
          <span>Kill</span>
        </div>
      );
    case "dim":
      return (
        <div
          className={cn(
            "flex items-center gap-1 text-muted-foreground whitespace-nowrap",
            className,
          )}
        >
          <SunIcon className="size-[0.75lh] shrink-0 text-amber-600 dark:text-amber-400" />
          <span>Dim over</span>
          <DurationText ticks={action.duration ?? 0} />
        </div>
      );
    case "message":
      return (
        <div
          className={cn(
            "flex items-center gap-1.5 text-muted-foreground min-w-0 max-w-50",
            className,
          )}
        >
          <MessageSquareIcon className="size-[0.75lh] shrink-0 text-blue-600 dark:text-blue-400" />
          <Text className="text-muted-foreground/70 min-w-0">
            {action.content}
          </Text>
        </div>
      );
    default:
      return null;
  }
}

export interface TriggerActionTextProps {
  action: TriggerAction;
  className?: string;
}

/**
 * Compact text-only trigger action display.
 * Used in list items and summary views.
 */
export function TriggerActionText({
  action,
  className,
}: TriggerActionTextProps) {
  return (
    <div className={cn("flex gap-1 items-center", className)}>
      <span>
        {action.tag === "dim"
          ? "Dim"
          : action.tag === "message"
            ? "Message"
            : "Kill"}
      </span>
      {action.tag === "dim" && (
        <div className="flex items-center">
          <span>(</span>
          <DurationText ticks={action.duration} />
          <span>)</span>
        </div>
      )}
      {action.tag === "message" && (
        <div className="flex items-center">
          <span>(</span>
          <Text className="max-w-24">{action.content}</Text>
          <span>)</span>
        </div>
      )}
    </div>
  );
}

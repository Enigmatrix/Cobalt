import { Badge } from "@/components/ui/badge";
import type { AlertTriggerStatus, ReminderTriggerStatus } from "@/lib/entities";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { BanIcon, BellIcon, CheckCircleIcon } from "lucide-react";

type StatusTag = "untriggered" | "hit" | "ignored";

const statusConfig = {
  untriggered: {
    label: "Untriggered",
    icon: CheckCircleIcon,
    className:
      "text-green-600 dark:text-green-400 border-green-600/50 dark:border-green-400/50 bg-green-500/10",
  },
  hit: {
    label: "Triggered",
    icon: BellIcon,
    className:
      "text-red-600 dark:text-red-400 border-red-600/50 dark:border-red-400/50 bg-red-500/10",
  },
  ignored: {
    label: "Ignored",
    icon: BanIcon,
    className:
      "text-muted-foreground border-muted-foreground/50 bg-muted-foreground/10",
  },
} as const;

// Reminder uses amber for "hit" instead of destructive
const reminderStatusConfig = {
  ...statusConfig,
  hit: {
    label: "Triggered",
    icon: BellIcon,
    className:
      "text-amber-600 dark:text-amber-400 border-amber-600/50 dark:border-amber-400/50 bg-amber-500/10",
  },
} as const;

export interface StatusBadgeProps {
  status: AlertTriggerStatus | ReminderTriggerStatus;
  /** Use reminder styling (amber for hit) */
  variant?: "alert" | "reminder";
  /** Additional className for badge */
  className?: ClassValue;
  /** Show icon */
  showIcon?: boolean;
  /** Show label */
  showLabel?: boolean;
}

export function StatusBadge({
  status,
  variant = "alert",
  className,
  showIcon = true,
  showLabel = true,
}: StatusBadgeProps) {
  const config = variant === "reminder" ? reminderStatusConfig : statusConfig;
  const { label, icon: Icon, className: statusClassName } = config[status.tag];

  return (
    <Badge
      variant="outline"
      className={cn(statusClassName, "gap-1", !showLabel && "p-1", className)}
    >
      {showIcon && <Icon className="size-3 shrink-0" />}
      {showLabel && label}
    </Badge>
  );
}

/** Get just the status label text */
export function getStatusLabel(status: { tag: StatusTag }): string {
  return statusConfig[status.tag].label;
}

import { Badge } from "@/components/ui/badge";
import type { AlertTriggerStatus, ReminderTriggerStatus } from "@/lib/entities";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { AlertCircleIcon, CheckCircleIcon, EyeOffIcon } from "lucide-react";

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
    icon: AlertCircleIcon,
    className: "text-destructive border-destructive/50 bg-destructive/10",
  },
  ignored: {
    label: "Ignored",
    icon: EyeOffIcon,
    className:
      "text-muted-foreground border-muted-foreground/50 bg-muted-foreground/10",
  },
} as const;

// Reminder uses amber for "hit" instead of destructive
const reminderStatusConfig = {
  ...statusConfig,
  hit: {
    label: "Triggered",
    icon: AlertCircleIcon,
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
}

export function StatusBadge({
  status,
  variant = "alert",
  className,
  showIcon = true,
}: StatusBadgeProps) {
  const config = variant === "reminder" ? reminderStatusConfig : statusConfig;
  const { label, icon: Icon, className: statusClassName } = config[status.tag];

  return (
    <Badge variant="outline" className={cn(statusClassName, className)}>
      {showIcon && <Icon className="w-3 h-3 mr-1" />}
      {label}
    </Badge>
  );
}

/** Get just the status label text */
export function getStatusLabel(status: { tag: StatusTag }): string {
  return statusConfig[status.tag].label;
}

import { StatusBadge } from "@/components/alert/status-badge";
import { TriggerActionIndicator } from "@/components/alert/trigger-action";
import AppIcon from "@/components/app/app-icon";
import { DurationText } from "@/components/time/duration-text";
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "@/components/ui/hover-card";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import { Text } from "@/components/ui/text";
import { useApp, useTag } from "@/hooks/use-refresh";
import type { Alert } from "@/lib/entities";
import { timeFrameToLabel } from "@/lib/entities";
import { ArrowRightIcon, ClockAlertIcon, TagIcon } from "lucide-react";
import { useMemo, type ReactNode } from "react";
import { NavLink } from "react-router";

export function AlertHoverCard({
  alert,
  children,
}: {
  alert: Alert;
  children: ReactNode;
}) {
  return (
    <HoverCard>
      <HoverCardTrigger asChild>{children}</HoverCardTrigger>
      <HoverCardContent side="bottom" align="start" className="w-80">
        <AlertHoverCardContent alert={alert} />
      </HoverCardContent>
    </HoverCard>
  );
}

function AlertHoverCardContent({ alert }: { alert: Alert }) {
  const app = useApp(alert.target.tag === "app" ? alert.target.id : null);
  const tag = useTag(alert.target.tag === "tag" ? alert.target.id : null);
  const targetEntity = app ?? tag;

  const currentUsage = useMemo(() => {
    const usages = alert.target.tag === "app" ? app?.usages : tag?.usages;
    if (!usages) return 0;
    switch (alert.timeFrame) {
      case "daily":
        return usages.today;
      case "weekly":
        return usages.week;
      case "monthly":
        return usages.month;
    }
  }, [alert, app, tag]);

  const progress = Math.min(
    (currentUsage / (alert.usageLimit || 1)) * 100,
    100,
  );

  const targetLink = app ? `/apps/${app.id}` : tag ? `/tags/${tag.id}` : "#";

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-3">
        {app ? (
          <AppIcon app={app} className="w-10 h-10 shrink-0" />
        ) : tag ? (
          <TagIcon
            className="w-10 h-10 shrink-0"
            style={{ color: tag.color }}
          />
        ) : null}
        <div className="flex flex-col min-w-0 gap-1">
          <NavLink
            to={targetLink}
            className="hover:underline"
            onClick={(e) => e.stopPropagation()}
          >
            <Text className="font-semibold">
              {targetEntity?.name ?? "Unknown"}
            </Text>
          </NavLink>
          <div className="flex items-center gap-2">
            <StatusBadge status={alert.status} className="text-xs h-5" />
            <TriggerActionIndicator
              action={alert.triggerAction}
              className="text-xs"
            />
          </div>
        </div>
      </div>

      <div className="space-y-1.5">
        <div className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-1.5">
            <DurationText ticks={currentUsage} className="font-medium" />
            <span className="text-muted-foreground">
              ({Math.round(progress)}%)
            </span>
          </div>
          <div className="flex items-center gap-1 text-muted-foreground text-xs">
            <DurationText ticks={alert.usageLimit} />
            <span>/</span>
            <span>{timeFrameToLabel(alert.timeFrame)}</span>
          </div>
        </div>
        <Progress value={progress} className="h-1.5 rounded-sm" />
      </div>

      {alert.reminders.length > 0 && (
        <>
          <Separator />
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <ClockAlertIcon className="size-3.5" />
            <span>
              {alert.reminders.length} Reminder
              {alert.reminders.length !== 1 ? "s" : ""}
            </span>
          </div>
        </>
      )}

      {targetEntity && (
        <>
          <Separator />
          <div className="grid grid-cols-3 gap-2 text-center">
            <div>
              <div className="text-xs text-muted-foreground">Today</div>
              <DurationText
                ticks={targetEntity.usages.today}
                className="text-sm font-medium"
              />
            </div>
            <div>
              <div className="text-xs text-muted-foreground">Week</div>
              <DurationText
                ticks={targetEntity.usages.week}
                className="text-sm font-medium"
              />
            </div>
            <div>
              <div className="text-xs text-muted-foreground">Month</div>
              <DurationText
                ticks={targetEntity.usages.month}
                className="text-sm font-medium"
              />
            </div>
          </div>
        </>
      )}

      <Separator />

      <NavLink
        to={`/alerts/${alert.id}`}
        className="text-sm text-primary hover:underline inline-flex items-center gap-1 self-start"
        onClick={(e) => e.stopPropagation()}
      >
        View Details
        <ArrowRightIcon className="size-3" />
      </NavLink>
    </div>
  );
}

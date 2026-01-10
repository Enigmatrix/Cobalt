import { StatusBadge } from "@/components/alert/status-badge";
import { TriggerActionIndicator } from "@/components/alert/trigger-action";
import AppIcon from "@/components/app/app-icon";
import { DateRangePicker } from "@/components/time/date-range-picker";
import { DurationText } from "@/components/time/duration-text";
import { DateTimeText } from "@/components/time/time-text";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button, buttonVariants } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { Text } from "@/components/ui/text";
import { NextButton, PrevButton } from "@/components/usage-card";
import {
  VizCard,
  VizCardAction,
  VizCardContent,
  VizCardHeader,
  VizCardTitle,
} from "@/components/viz/viz-card";
import { useAlert, useApp, useTag } from "@/hooks/use-refresh";
import { useAlertEvents, useAlertReminderEvents } from "@/hooks/use-repo";
import { useIntervalControlsWithDefault } from "@/hooks/use-time";
import type { Alert, App, Ref, Tag } from "@/lib/entities";
import { timeFrameToLabel } from "@/lib/entities";
import { useAppState } from "@/lib/state";
import { ticksToDateTime } from "@/lib/time";
import { cn } from "@/lib/utils";
import _ from "lodash";
import {
  Ban,
  BellIcon,
  BellOffIcon,
  ClockAlert,
  ClockIcon,
  Edit2Icon,
  TagIcon,
  TrashIcon,
} from "lucide-react";
import { useCallback, useMemo } from "react";
import { NavLink, useNavigate } from "react-router";
import type { Route } from "../alerts/+types/[id]";

export default function Page({ params }: Route.ComponentProps) {
  const id = +params.id as Ref<Alert>;
  const alert = useAlert(id);
  if (!alert) return null;
  return <AlertPage alert={alert} />;
}

function AlertPage({ alert }: { alert: Alert }) {
  const removeAlert = useAppState((state) => state.removeAlert);
  const ignoreAlert = useAppState((state) => state.ignoreAlert);
  const navigate = useNavigate();
  const remove = useCallback(async () => {
    await navigate("/alerts");
    await removeAlert(alert.id);
  }, [removeAlert, navigate, alert.id]);

  // Get target entity (app or tag)
  const app = useApp(alert.target.tag === "app" ? alert.target.id : null);
  const tag = useTag(alert.target.tag === "tag" ? alert.target.id : null);
  const targetEntity = app ?? tag;
  const targetName = targetEntity?.name ?? "Unknown";

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb className="overflow-hidden">
          <BreadcrumbList className="flex-nowrap overflow-hidden">
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbLink asChild>
                <NavLink to="/alerts">Alerts</NavLink>
              </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem className="overflow-hidden">
              <BreadcrumbPage className="inline-flex items-center overflow-hidden">
                {app && (
                  <AppIcon
                    appIcon={app.icon}
                    className="w-5 h-5 mr-2 shrink-0"
                  />
                )}
                {tag && (
                  <TagIcon
                    className="w-5 h-5 mr-2 shrink-0"
                    style={{ color: tag.color }}
                  />
                )}
                <Text className="truncate">{targetName}</Text>
              </BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>

      <div className="h-0 flex-auto overflow-auto [scrollbar-gutter:stable]">
        <div className="flex flex-col gap-4 p-4">
          {/* Alert Configuration */}
          <AlertInfoCard
            alert={alert}
            app={app}
            tag={tag}
            onRemove={remove}
            onIgnore={() => ignoreAlert(alert.id)}
          />

          {/* Reminders */}
          <RemindersCard alert={alert} />

          {/* Alert Timeline */}
          <AlertTimelineCard alert={alert} />
        </div>
      </div>
    </>
  );
}

/** Get current usage for an alert based on its timeFrame */
function getCurrentUsage(
  alert: Alert,
  app: App | null,
  tag: Tag | null,
): number {
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
}

function AlertInfoCard({
  alert,
  app,
  tag,
  onRemove,
  onIgnore,
}: {
  alert: Alert;
  app: App | null;
  tag: Tag | null;
  onRemove: () => void;
  onIgnore: () => void;
}) {
  const targetEntity = app ?? tag;
  const currentUsage = useMemo(
    () => getCurrentUsage(alert, app, tag),
    [alert, app, tag],
  );
  const limit = alert.usageLimit;
  const progress = Math.min((currentUsage / (limit || 1)) * 100, 100);
  const targetLink = app ? `/apps/${app.id}` : tag ? `/tags/${tag.id}` : "#";

  return (
    <div className="rounded-xl bg-card border border-border px-6 pt-6 pb-4">
      <div className="flex flex-col gap-4">
        {/* Header with name and icon */}
        <div className="flex items-center gap-4">
          <NavLink to={targetLink}>
            {app ? (
              <AppIcon
                appIcon={app.icon}
                className="w-12 h-12 shrink-0 hover:opacity-80 transition-opacity"
              />
            ) : tag ? (
              <TagIcon
                className="w-12 h-12 shrink-0 hover:opacity-80 transition-opacity"
                style={{ color: tag.color }}
              />
            ) : null}
          </NavLink>
          <div className="min-w-0 shrink flex flex-col gap-1">
            <NavLink
              to={targetLink}
              className="hover:opacity-80 transition-opacity"
            >
              <Text className="text-2xl font-semibold">
                {targetEntity?.name ?? "Unknown"}
              </Text>
            </NavLink>
            <StatusBadge status={alert.status} />
          </div>
          <div className="flex-1" />
          <AlertActions alert={alert} onRemove={onRemove} onIgnore={onIgnore} />
        </div>

        {/* Progress bar with usage */}
        <UsageProgressBar
          currentUsage={currentUsage}
          limit={limit}
          progress={progress}
          timeFrameLabel={timeFrameToLabel(alert.timeFrame)}
          action={alert.triggerAction}
        />
      </div>
    </div>
  );
}

function AlertActions({
  alert,
  onRemove,
  onIgnore,
}: {
  alert: Alert;
  onRemove: () => void;
  onIgnore: () => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <Button
        variant="outline"
        size="sm"
        disabled={alert.status.tag === "ignored"}
        onClick={onIgnore}
      >
        <Ban className="w-4 h-4 mr-1" />
        Ignore
      </Button>
      <Button variant="outline" size="icon" asChild>
        <NavLink to={`/alerts/edit/${alert.id}`}>
          <Edit2Icon className="w-4 h-4" />
        </NavLink>
      </Button>
      <AlertDialog>
        <AlertDialogTrigger asChild>
          <Button size="icon" variant="outline">
            <TrashIcon className="w-4 h-4" />
          </Button>
        </AlertDialogTrigger>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Remove Alert?</AlertDialogTitle>
            <AlertDialogDescription>
              This action cannot be undone. All alert history will be removed.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={onRemove}
              className={buttonVariants({ variant: "destructive" })}
            >
              Remove
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

function UsageProgressBar({
  currentUsage,
  limit,
  progress,
  timeFrameLabel,
  action,
}: {
  currentUsage: number;
  limit: number;
  progress: number;
  timeFrameLabel: string;
  action: Alert["triggerAction"];
}) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <DurationText
            ticks={currentUsage}
            className="font-semibold"
            symbolForZero="No Usage"
          />
          <span className="text-muted-foreground">
            ({Math.round(progress)}%)
          </span>
          <span className="text-muted-foreground">/</span>
          <DurationText ticks={limit} className="text-muted-foreground" />
          <span className="text-muted-foreground">/</span>
          <span className="text-muted-foreground">{timeFrameLabel}</span>
        </div>
        <TriggerActionIndicator action={action} />
      </div>
      <Progress value={progress} className="h-2 rounded-sm" />
    </div>
  );
}

function RemindersCard({ alert }: { alert: Alert }) {
  return (
    <VizCard>
      <VizCardHeader className="pb-4">
        <VizCardTitle className="pl-4 pt-4 font-semibold">
          Reminders
        </VizCardTitle>
      </VizCardHeader>
      <VizCardContent className="px-4 pb-4">
        {alert.reminders.length === 0 ? (
          <div className="text-center py-6 text-muted-foreground">
            <ClockAlert className="w-8 h-8 mx-auto mb-2 opacity-50" />
            <Text className="text-sm">No reminders configured</Text>
            <Text className="text-xs mt-1">
              Edit this alert to add reminders
            </Text>
          </div>
        ) : (
          <div className="space-y-3">
            {_(alert.reminders)
              .sortBy((r) => r.threshold)
              .map((reminder) => {
                const triggerDuration = reminder.threshold * alert.usageLimit;

                return (
                  <div
                    key={reminder.id}
                    className="flex items-start gap-3 p-3 rounded-lg border border-border bg-muted/20"
                  >
                    <ClockAlert className="w-4 h-4 text-muted-foreground shrink-0 mt-0.5" />
                    <div className="flex-1 min-w-0 space-y-1">
                      <Text className="font-medium leading-snug">
                        {reminder.message}
                      </Text>
                      <div className="flex items-center gap-3 flex-wrap">
                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                          <Text>{`${Math.round(reminder.threshold * 100)}%`}</Text>
                          <Text>-</Text>
                          <DurationText ticks={triggerDuration} />
                        </div>
                        <StatusBadge
                          status={reminder.status}
                          variant="reminder"
                          className="text-xs h-5"
                          showIcon={false}
                        />
                      </div>
                    </div>
                  </div>
                );
              })
              .value()}
          </div>
        )}
      </VizCardContent>
    </VizCard>
  );
}

function AlertTimelineCard({ alert }: { alert: Alert }) {
  const { interval, canGoNext, goNext, canGoPrev, goPrev, setInterval } =
    useIntervalControlsWithDefault("week");

  const { ret: alertEvents, isLoading: eventsLoading } = useAlertEvents({
    start: interval.start,
    end: interval.end,
    alertId: alert.id,
  });

  const { ret: reminderEvents, isLoading: reminderLoading } =
    useAlertReminderEvents({
      start: interval.start,
      end: interval.end,
      alertId: alert.id,
    });

  const isLoading = eventsLoading || reminderLoading;

  // Combine and sort events chronologically
  const timelineEvents = useMemo(() => {
    const combined = [
      ...alertEvents.map((e) => ({ ...e, type: "alert" as const })),
      ...reminderEvents.map((e) => ({ ...e, type: "reminder" as const })),
    ];

    return _(combined)
      .sortBy((e) => -e.timestamp)
      .value(); // Most recent first
  }, [alertEvents, reminderEvents]);

  return (
    <VizCard>
      <VizCardHeader className="pb-4">
        <VizCardTitle className="pl-4 pt-4 flex items-center gap-2">
          <span className="font-semibold">Events</span>
          {isLoading && (
            <div className="w-4 h-4 border-2 border-muted border-t-primary rounded-full animate-spin" />
          )}
        </VizCardTitle>
        <VizCardAction className="mt-4 mr-1.5 flex">
          <PrevButton
            canGoPrev={canGoPrev}
            isLoading={isLoading}
            goPrev={goPrev}
          />
          <DateRangePicker
            className="min-w-32"
            value={interval}
            onChange={setInterval}
          />
          <NextButton
            canGoNext={canGoNext}
            isLoading={isLoading}
            goNext={goNext}
          />
        </VizCardAction>
      </VizCardHeader>
      <VizCardContent className="px-4 pb-4">
        {timelineEvents.length === 0 ? (
          <div className="text-center py-6 text-muted-foreground">
            <ClockIcon className="w-8 h-8 mx-auto mb-2 opacity-50" />
            <Text className="text-sm">No events in this period</Text>
          </div>
        ) : (
          <div className="relative">
            {/* Timeline line */}
            <div className="absolute left-3.5 top-0 bottom-0 w-px bg-border" />

            {timelineEvents.map((event, index) => {
              const isIgnored = event.reason !== "hit";
              const isAlert = event.type === "alert";

              // Check if this event is on a different day than the previous one
              const prevEvent = index > 0 ? timelineEvents[index - 1] : null;
              const currentDay = ticksToDateTime(event.timestamp).startOf(
                "day",
              );
              const prevDay = prevEvent
                ? ticksToDateTime(prevEvent.timestamp).startOf("day")
                : null;
              const isDifferentDay =
                prevDay !== null && !currentDay.equals(prevDay);

              return (
                <div
                  key={`${event.type}-${event.id}`}
                  className={cn(
                    "relative flex items-center gap-3 pb-3",
                    isDifferentDay && "mt-5",
                  )}
                >
                  {/* Timeline dot */}
                  <div
                    className={cn(
                      "relative z-10 flex h-9 w-9 items-center justify-center rounded-full outline-1 bg-card",
                      isIgnored
                        ? "outline-zinc-500 dark:outline-zinc-400 text-zinc-500 dark:text-zinc-400"
                        : isAlert
                          ? "outline-red-600 dark:outline-red-400 text-red-600 dark:text-red-400"
                          : "outline-amber-600 dark:outline-amber-400 text-amber-600 dark:text-amber-400",
                    )}
                  >
                    {isAlert ? (
                      isIgnored ? (
                        <BellOffIcon className="size-5" />
                      ) : (
                        <BellIcon className="size-5" />
                      )
                    ) : (
                      <ClockAlert className="size-5" />
                    )}
                  </div>

                  {/* Event content */}
                  <div className="flex-1 min-w-0 flex flex-col gap-0.5">
                    <div className="text-xs text-muted-foreground flex">
                      <DateTimeText ticks={event.timestamp} />
                    </div>
                    <div className="flex items-center gap-2 min-w-0">
                      {isAlert ? (
                        <span
                          className={cn(
                            "text-sm font-medium",
                            isIgnored
                              ? "text-zinc-600 dark:text-zinc-300"
                              : "text-red-600 dark:text-red-400",
                          )}
                        >
                          {isIgnored ? "Ignored" : "Triggered"}
                        </span>
                      ) : (
                        <>
                          <span
                            className={cn(
                              "text-sm font-medium shrink-0",
                              isIgnored
                                ? "text-zinc-500 dark:text-zinc-400"
                                : "text-amber-600 dark:text-amber-400",
                            )}
                          >
                            {Math.round(event.threshold * 100)}%
                          </span>
                          <Text
                            className={cn(
                              "text-sm min-w-0",
                              isIgnored
                                ? "text-zinc-500 dark:text-zinc-400"
                                : "text-foreground",
                            )}
                          >
                            {event.message}
                          </Text>
                        </>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </VizCardContent>
    </VizCard>
  );
}

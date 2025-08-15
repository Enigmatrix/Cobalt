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
import { Badge } from "@/components/ui/badge";
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
import type { Alert, App, Ref, Tag, TriggerAction } from "@/lib/entities";
import { useAppState } from "@/lib/state";
import { cn } from "@/lib/utils";
import _ from "lodash";
import {
  AlertCircleIcon,
  BellIcon,
  CheckCircleIcon,
  ClockAlert,
  ClockIcon,
  Edit2Icon,
  EyeOffIcon,
  MessageSquareIcon,
  SunDimIcon,
  TagIcon,
  TrashIcon,
  XCircleIcon,
  ZapIcon,
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
          <AlertInfoCard alert={alert} app={app} tag={tag} onRemove={remove} />

          {/* Current Progress */}
          <CurrentProgressCard alert={alert} />

          {/* Reminders */}
          <RemindersCard alert={alert} />

          {/* Alert Timeline */}
          <AlertTimelineCard alert={alert} />
        </div>
      </div>
    </>
  );
}

function AlertInfoCard({
  alert,
  app,
  tag,
  onRemove,
}: {
  alert: Alert;
  app: App | null;
  tag: Tag | null;
  onRemove: () => void;
}) {
  const targetEntity = app ?? tag;
  const timeFrameText =
    alert.timeFrame === "daily"
      ? "Daily"
      : alert.timeFrame === "weekly"
        ? "Weekly"
        : "Monthly";

  return (
    <VizCard>
      <VizCardContent className="p-6">
        <div className="flex flex-col gap-6">
          {/* Header */}
          <div className="flex items-start gap-4">
            <div className="flex items-center gap-4">
              {app && (
                <NavLink to={`/apps/${app.id}`}>
                  <AppIcon
                    appIcon={app.icon}
                    className="w-12 h-12 shrink-0 hover:opacity-80 transition-opacity"
                  />
                </NavLink>
              )}
              {tag && (
                <NavLink to={`/tags/${tag.id}`}>
                  <TagIcon
                    className="w-12 h-12 shrink-0 hover:opacity-80 transition-opacity"
                    style={{ color: tag.color }}
                  />
                </NavLink>
              )}
              <div className="flex flex-col gap-2">
                <NavLink
                  to={app ? `/apps/${app.id}` : tag ? `/tags/${tag.id}` : "#"}
                  className="hover:opacity-80 transition-opacity"
                >
                  <Text className="text-2xl font-semibold">
                    {targetEntity?.name ?? "Unknown"}
                  </Text>
                </NavLink>
                <StatusBadge status={alert.status} />
              </div>
            </div>
            <div className="flex-1" />
            <div className="flex items-center gap-2">
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
                      This action cannot be undone. All alert history will be
                      removed.
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
          </div>

          {/* Configuration Details */}
          <div className="grid gap-4 md:grid-cols-3">
            <div className="space-y-2">
              <Text className="text-sm font-medium text-muted-foreground">
                Usage Limit
              </Text>
              <div className="flex items-center gap-2">
                <ClockIcon className="w-4 h-4 text-muted-foreground" />
                <DurationText ticks={alert.usageLimit} />
              </div>
            </div>
            <div className="space-y-2">
              <Text className="text-sm font-medium text-muted-foreground">
                Time Frame
              </Text>
              <Text>{timeFrameText}</Text>
            </div>
            <div className="space-y-2">
              <Text className="text-sm font-medium text-muted-foreground">
                Action
              </Text>
              <TriggerActionDisplay action={alert.triggerAction} />
            </div>
          </div>
        </div>
      </VizCardContent>
    </VizCard>
  );
}

function StatusBadge({ status }: { status: Alert["status"] }) {
  switch (status.tag) {
    case "untriggered":
      return (
        <Badge
          variant="outline"
          className="text-green-500/80 border-green-500/50 bg-green-500/10"
        >
          <CheckCircleIcon className="w-3 h-3 mr-1" />
          Untriggered
        </Badge>
      );
    case "hit":
      return (
        <Badge
          variant="outline"
          className="text-red-500/80 border-red-500/50 bg-red-500/10"
        >
          <AlertCircleIcon className="w-3 h-3 mr-1" />
          Triggered
        </Badge>
      );
    case "ignored":
      return (
        <Badge
          variant="outline"
          className="text-gray-600/80 border-gray-600/20 bg-gray-50/50"
        >
          <EyeOffIcon className="w-3 h-3 mr-1" />
          Ignored
        </Badge>
      );
    default:
      return null;
  }
}

function TriggerActionDisplay({ action }: { action: TriggerAction }) {
  switch (action.tag) {
    case "kill":
      return (
        <div className="flex items-center gap-2">
          <ZapIcon className="w-4 h-4 shrink-0 text-red-500" />
          <Text>Kill</Text>
        </div>
      );
    case "dim":
      return (
        <div className="flex items-center gap-2">
          <SunDimIcon className="w-4 h-4 shrink-0 text-orange-500" />
          <div className="flex items-center gap-1">
            <Text>Dim over</Text>
            <DurationText ticks={action.duration ?? 0} />
          </div>
        </div>
      );
    case "message":
      return (
        <div className="flex items-center gap-2">
          <MessageSquareIcon className="w-4 h-4 shrink-0 text-blue-500" />
          <Text>{action.content}</Text>
        </div>
      );
    default:
      return <Text>Unknown Action</Text>;
  }
}

function CurrentProgressCard({ alert }: { alert: Alert }) {
  const app = useApp(alert.target.tag === "app" ? alert.target.id : null);
  const tag = useTag(alert.target.tag === "tag" ? alert.target.id : null);

  const currentUsage = useMemo(() => {
    const usages = alert.target.tag === "app" ? app?.usages : tag?.usages;
    switch (alert.timeFrame) {
      case "daily":
        return usages!.today;
      case "weekly":
        return usages!.week;
      case "monthly":
        return usages!.month;
    }
  }, [app, tag, alert]);

  const limit = alert.usageLimit;
  const progress = limit > 0 ? Math.min((currentUsage / limit) * 100, 100) : 0;

  return (
    <VizCard>
      <VizCardHeader className="px-4 pt-4">
        <VizCardTitle className="flex items-center gap-2 mt-2 ml-2 font-semibold">
          Current Progress
        </VizCardTitle>
        <VizCardAction className="flex items-center">
          <div className="flex items-center gap-1 text-sm text-muted-foreground mr-2 mt-2">
            <DurationText ticks={currentUsage ?? 0} />
            <Text>/</Text>
            <DurationText ticks={limit ?? 0} />
          </div>
        </VizCardAction>
      </VizCardHeader>
      <VizCardContent className="p-4 px-6">
        <div className="space-y-4">
          <Progress value={progress} className="h-3 rounded-sm" />
          <div className="flex justify-between text-sm text-muted-foreground">
            <Text>0%</Text>
            <Text>{`${Math.round(progress)}%`}</Text>
            <Text>100%</Text>
          </div>
        </div>
      </VizCardContent>
    </VizCard>
  );
}

function RemindersCard({ alert }: { alert: Alert }) {
  return (
    <VizCard>
      <VizCardHeader className="px-4 pt-4">
        <VizCardTitle className="flex items-center gap-2 mt-2 ml-2 font-semibold">
          Reminders
        </VizCardTitle>
      </VizCardHeader>
      <VizCardContent className="p-4">
        <div>
          {alert.reminders.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <ClockAlert className="w-8 h-8 mx-auto mb-2 opacity-50" />
              <Text className="text-sm">No reminders configured</Text>
              <Text className="text-xs mt-1">
                Edit this alert to add reminders
              </Text>
            </div>
          ) : (
            <div className="space-y-4">
              {_(alert.reminders)
                .sortBy((r) => r.threshold)
                .map((reminder) => {
                  const triggerDuration = reminder.threshold * alert.usageLimit;

                  return (
                    <div
                      key={reminder.id}
                      className="flex items-start gap-3 p-3 rounded-lg border border-border bg-muted/20"
                    >
                      <ClockAlert className="w-4 h-4 text-muted-foreground shrink-0 mt-1" />
                      <div className="flex-1 min-w-0 space-y-1.5">
                        {/* Primary message */}
                        <Text className="font-medium leading-snug">
                          {reminder.message}
                        </Text>

                        {/* Secondary info */}
                        <div className="flex items-center gap-3 flex-wrap">
                          <div className="flex items-center gap-2 text-xs text-muted-foreground">
                            <Text>{`${Math.round(reminder.threshold * 100)}%`}</Text>
                            <Text>·</Text>
                            <DurationText ticks={triggerDuration} />
                          </div>

                          <Badge
                            variant="outline"
                            className={cn(
                              "text-xs h-5",
                              reminder.status.tag === "untriggered" &&
                                "text-green-500/80 border-green-500/50 bg-green-500/10",
                              reminder.status.tag === "hit" &&
                                "text-orange-500/80 border-orange-500/50 bg-orange-500/10",
                              reminder.status.tag === "ignored" &&
                                "text-gray-500/80 border-gray-500/50 bg-gray-500/10",
                            )}
                          >
                            {reminder.status.tag === "untriggered"
                              ? "Untriggered"
                              : reminder.status.tag === "hit"
                                ? "Triggered"
                                : "Ignored"}
                          </Badge>
                        </div>
                      </div>
                    </div>
                  );
                })
                .value()}
            </div>
          )}
        </div>
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
      .sortBy((e) => e.timestamp)
      .value(); // Most recent first
  }, [alertEvents, reminderEvents]);

  return (
    <VizCard>
      <VizCardHeader className="p-4">
        <VizCardTitle className="flex items-center gap-2 mt-2 ml-2">
          <div className="font-semibold">Events</div>
          {isLoading && (
            <div className="w-4 h-4 border-2 border-muted border-t-blue-500 rounded-full animate-spin" />
          )}
        </VizCardTitle>
        <VizCardAction className="flex items-center">
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
      <VizCardContent className="px-6">
        <div>
          {timelineEvents.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              <ClockIcon className="w-8 h-8 mx-auto mb-2 opacity-50" />
              <Text className="text-sm">No events in this period</Text>
            </div>
          ) : (
            <div className="relative">
              {/* Timeline line */}
              <div className="absolute left-4 top-0 bottom-0 w-px bg-border" />

              {timelineEvents.map((event) => (
                <div
                  key={`${event.type}-${event.id}`}
                  className="relative flex items-start gap-4 pb-4"
                >
                  {/* Timeline dot */}
                  <div
                    className={cn(
                      "relative z-10 flex h-8 w-8 items-center justify-center rounded-full outline-1 bg-card",
                      event.type === "alert"
                        ? event.reason === "hit"
                          ? "outline-orange-400 text-orange-400"
                          : "outline-gray-300 text-gray-300"
                        : event.reason === "hit"
                          ? "outline-yellow-300 text-yellow-300"
                          : "outline-gray-300 text-gray-300",
                    )}
                  >
                    {event.type === "alert" ? (
                      event.reason === "hit" ? (
                        <BellIcon className="w-4 h-4" />
                      ) : (
                        <XCircleIcon className="w-4 h-4" />
                      )
                    ) : event.reason === "hit" ? (
                      <ClockAlert className="w-4 h-4" />
                    ) : (
                      <XCircleIcon className="w-4 h-4" />
                    )}
                  </div>

                  {/* Event content */}
                  <div className="flex-1 min-w-0 pb-4">
                    <div className="flex items-center justify-start gap-2 mb-1">
                      <Text className="text-sm font-medium">
                        {event.type === "alert"
                          ? `Alert${event.reason === "hit" ? "" : " Ignored"}`
                          : `Reminder${event.reason === "hit" ? "" : " Ignored"}`}
                      </Text>
                    </div>
                    <DateTimeText
                      ticks={event.timestamp}
                      className="text-xs text-muted-foreground w-fit"
                    />
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </VizCardContent>
    </VizCard>
  );
}

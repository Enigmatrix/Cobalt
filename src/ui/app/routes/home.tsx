import AppIcon from "@/components/app/app-icon";
import { ScoreCircle } from "@/components/tag/score";
import { DateRangePicker } from "@/components/time/date-range-picker";
import { DurationText } from "@/components/time/duration-text";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
} from "@/components/ui/breadcrumb";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { Text } from "@/components/ui/text";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { NextButton, PrevButton, UsageCard } from "@/components/usage-card";
import { Gantt } from "@/components/viz/gantt2";
import { UsageChart } from "@/components/viz/usage-chart";
import {
  VizCard,
  VizCardAction,
  VizCardContent,
  VizCardHeader,
  VizCardTitle,
} from "@/components/viz/viz-card";
import { useAlerts, useApp, useTag } from "@/hooks/use-refresh";
import {
  useAppDurationsPerPeriod,
  useAppSessionUsages,
  useDefaultStreaks,
  useInteractionPeriods,
  useStreakDurations,
  useSystemEvents,
  useTotalUsageFromPerPeriod,
} from "@/hooks/use-repo";
import {
  useIntervalControlsWithDefault,
  usePeriodInterval,
} from "@/hooks/use-time";
import type { Alert, TimeFrame } from "@/lib/entities";
import {
  hour24Formatter,
  monthDayFormatter,
  weekDayFormatter,
  type Period,
} from "@/lib/time";
import { cn } from "@/lib/utils";
import _ from "lodash";
import { TagIcon } from "lucide-react";
import { DateTime } from "luxon";
import { useMemo } from "react";

export default function Home() {
  const interval = usePeriodInterval("day");
  const { ret: streaks } = useDefaultStreaks(interval);
  const { ret: appDurationsPerPeriod } = useAppDurationsPerPeriod({
    ...interval,
    period: "hour",
  });

  const [focusStreakUsage, distractiveStreakUsage] =
    useStreakDurations(streaks);
  const totalUsage = useTotalUsageFromPerPeriod(appDurationsPerPeriod);

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbPage>Home</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>
      <div className="h-0 flex-auto overflow-y-auto overflow-x-hidden [scrollbar-gutter:stable]">
        <div className="flex flex-col gap-4 p-4">
          <div className="grid auto-rows-min gap-4 grid-cols-4">
            <AlertEventItem />
            <UsageStatItem
              label="Focus Streaks"
              usage={focusStreakUsage}
              color="text-green-600 dark:text-green-400"
              cardColor="bg-green-500/10 border-green-500/20"
            />
            <UsageStatItem
              label="Distractive Streaks"
              usage={distractiveStreakUsage}
              color="text-red-600 dark:text-red-400"
              cardColor="bg-red-500/10 border-red-500/20"
            />
            <UsageStatItem
              label="Total Usage"
              usage={totalUsage}
              color="text-foreground"
            />
          </div>

          <div className="grid auto-rows-min gap-4 grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
            <AppUsageBarChartCard
              timePeriod="day"
              period="hour"
              xAxisLabelFormatter={hour24Formatter}
            />
            <AppUsageBarChartCard
              timePeriod="week"
              period="day"
              xAxisLabelFormatter={weekDayFormatter}
            />
          </div>

          <div className="grid auto-rows-min gap-4 grid-cols-1">
            <AppUsageBarChartCard
              timePeriod="month"
              period="day"
              xAxisLabelFormatter={monthDayFormatter}
            />
          </div>

          <SessionsCard />
        </div>
      </div>
    </>
  );
}

function AppUsageBarChartCard({
  timePeriod,
  period,
  xAxisLabelFormatter,
}: {
  timePeriod: Period;
  period: Period;
  xAxisLabelFormatter: (dt: DateTime) => string;
}) {
  const { interval, canGoNext, goNext, canGoPrev, goPrev } =
    useIntervalControlsWithDefault(timePeriod);

  const { isLoading, ret: appDurationsPerPeriod } = useAppDurationsPerPeriod({
    ...interval,
    period,
  });
  const totalUsage = useTotalUsageFromPerPeriod(appDurationsPerPeriod);

  const children = useMemo(
    () => (
      <div className="aspect-video flex-1 mx-1 max-w-full max-h-80">
        <UsageChart
          appDurationsPerPeriod={appDurationsPerPeriod}
          period={period}
          start={interval.start}
          end={interval.end}
          xAxisFormatter={xAxisLabelFormatter}
          className="aspect-none"
          maxYIsPeriod
          animationsEnabled={false}
          barRadius={2}
        />
      </div>
    ),
    [appDurationsPerPeriod, period, xAxisLabelFormatter, interval],
  );

  return (
    <UsageCard
      interval={interval}
      totalUsage={totalUsage}
      children={children}
      actions={
        <>
          <PrevButton
            canGoPrev={canGoPrev}
            isLoading={isLoading}
            goPrev={goPrev}
          />
          <NextButton
            canGoNext={canGoNext}
            isLoading={isLoading}
            goNext={goNext}
          />
        </>
      }
    />
  );
}

function SessionsCard() {
  const { interval, canGoNext, goNext, canGoPrev, goPrev, setInterval } =
    useIntervalControlsWithDefault("day");

  const { ret: usages, isLoading: usagesLoading } =
    useAppSessionUsages(interval);
  const { ret: interactions, isLoading: interactionPeriodsLoading } =
    useInteractionPeriods(interval);
  const { ret: systemEvents, isLoading: systemEventsLoading } =
    useSystemEvents(interval);
  const { ret: streaks, isLoading: streaksLoading } =
    useDefaultStreaks(interval);

  const isLoading =
    usagesLoading ||
    interactionPeriodsLoading ||
    systemEventsLoading ||
    streaksLoading;

  return (
    <VizCard>
      <VizCardHeader className="pb-1 has-data-[slot=card-action]:grid-cols-[minmax(0,1fr)_auto]">
        <VizCardTitle className="pl-4 pt-4 text-lg font-bold">
          Sessions
        </VizCardTitle>

        <VizCardAction className="flex mt-3 mr-1.5">
          {
            <>
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
            </>
          }
        </VizCardAction>
      </VizCardHeader>

      <VizCardContent>
        <Gantt
          usages={usages}
          usagesLoading={usagesLoading}
          streaks={streaks}
          streaksLoading={streaksLoading}
          interactionPeriods={interactions}
          interactionPeriodsLoading={interactionPeriodsLoading}
          systemEvents={systemEvents}
          systemEventsLoading={systemEventsLoading}
          interval={interval}
        />
      </VizCardContent>
    </VizCard>
  );
}

// Component for displaying usage statistics
function UsageStatItem({
  label,
  usage,
  color,
  cardColor,
}: {
  label: string;
  usage: number;
  color?: string;
  cardColor?: string;
}) {
  return (
    <div
      className={cn(
        "rounded-lg p-4 shadow-sm border text-right",
        cardColor ?? "bg-card border-border",
      )}
    >
      <div className="flex flex-col">
        <div className={cn("text-sm font-medium", color)}>{label}</div>
        <DurationText
          className={cn("text-xl font-bold self-end", color)}
          ticks={usage}
        />
      </div>
    </div>
  );
}

function AlertEventItem() {
  const alerts = useAlerts();
  const triggeredAlerts = useMemo(
    () =>
      _(alerts)
        .filter((alert) => alert.events.today > 0)
        .value(),
    [alerts],
  );

  return (
    <div className="rounded-lg p-4 shadow-sm border text-right bg-card border-border">
      <div className="flex flex-col">
        <div className="text-sm font-medium">Triggered Alerts</div>
        {triggeredAlerts.length > 0 ? (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger className="self-end">
                <div className="text-xl font-bold">
                  {triggeredAlerts.length}
                </div>
              </TooltipTrigger>
              <TooltipContent className="gap-2">
                {triggeredAlerts.map((alert) => (
                  <MiniAlertItem key={alert.id} alert={alert} />
                ))}
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        ) : (
          <div className="text-xl font-bold">None</div>
        )}
      </div>
    </div>
  );
}

function MiniAlertItem({ alert }: { alert: Alert }) {
  return (
    <div className="shadow-lg p-1 rounded-md bg-card border border-border flex flex-row gap-2 items-center">
      <MiniTargetItem alert={alert} />
      <div className="flex flex-row gap-1 items-center text-xs text-muted-foreground">
        <DurationText ticks={alert.usageLimit} />
        <div className="">{" / " + timeFrameToString(alert.timeFrame)}</div>
      </div>
      <MiniAlertTriggerAction alert={alert} />
    </div>
  );
}

function timeFrameToString(timeFrame: TimeFrame) {
  return timeFrame === "daily"
    ? "day"
    : timeFrame === "weekly"
      ? "week"
      : "month";
}

function MiniTargetItem({ alert }: { alert: Alert }) {
  const app = useApp(alert.target.tag === "app" ? alert.target.id : null);
  const tag = useTag(alert.target.tag === "tag" ? alert.target.id : null);

  return alert.target.tag === "app" && app ? (
    <div className="flex gap-1 items-center">
      <AppIcon className="h-4 w-4 shrink-0" appIcon={app.icon} />
      <div>{app.name}</div>
    </div>
  ) : alert.target.tag === "tag" && tag ? (
    <div className="flex gap-1 items-center">
      <TagIcon className="h-4 w-4 shrink-0" style={{ color: tag.color }} />
      <div>{tag.name}</div>
      <ScoreCircle score={tag.score} />
    </div>
  ) : null;
}

function MiniAlertTriggerAction({ alert }: { alert: Alert }) {
  return (
    <div className="flex gap-1 items-center text-xs text-muted-foreground">
      <span>
        {alert.triggerAction.tag === "dim"
          ? "Dim"
          : alert.triggerAction.tag === "message"
            ? "Message"
            : "Kill"}
      </span>
      {alert.triggerAction.tag === "dim" && (
        <div className="flex items-center">
          <span>(</span>
          <DurationText ticks={alert.triggerAction.duration} />
          <span>)</span>
        </div>
      )}
      {alert.triggerAction.tag === "message" && (
        <div className="flex items-center">
          <span>(</span>
          <Text className="max-w-24">{alert.triggerAction.content}</Text>
          <span>)</span>
        </div>
      )}
    </div>
  );
}

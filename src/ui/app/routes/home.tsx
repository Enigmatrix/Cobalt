import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
} from "@/components/ui/breadcrumb";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { NextButton, PrevButton, UsageCard } from "@/components/usage-card";
import { Gantt } from "@/components/viz/gantt2";
import { UsageChart } from "@/components/viz/usage-chart";
import { useLastNonNull } from "@/hooks/use-last";
import {
  useAppDurationsPerPeriod,
  useAppSessionUsages,
  useInteractionPeriods,
  useSystemEvents,
} from "@/hooks/use-repo";
import {
  useIntervalControlsWithDefault,
  usePeriodInterval,
} from "@/hooks/use-time";
import {
  hour24Formatter,
  monthDayFormatter,
  weekDayFormatter,
  type Period,
} from "@/lib/time";
import { DateTime } from "luxon";
import { useMemo } from "react";

export default function Home() {
  const interval = usePeriodInterval("day");

  const { ret: usages, isLoading: usagesLoading } = useAppSessionUsages({
    start: interval.start,
    end: interval.end,
  });
  const { ret: interactions, isLoading: interactionPeriodsLoading } =
    useInteractionPeriods({
      start: interval.start,
      end: interval.end,
    });
  const { ret: systemEvents, isLoading: systemEventsLoading } = useSystemEvents(
    {
      start: interval.start,
      end: interval.end,
    },
  );

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

          <div className="sticky rounded-xl bg-muted/50 border border-border overflow-clip">
            <Gantt
              usages={usages}
              usagesLoading={usagesLoading}
              interactionPeriods={interactions}
              interactionPeriodsLoading={interactionPeriodsLoading}
              systemEvents={systemEvents}
              systemEventsLoading={systemEventsLoading}
              interval={interval}
            />
          </div>
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
  const {
    interval: intervalNullable,
    canGoNext,
    goNext,
    canGoPrev,
    goPrev,
  } = useIntervalControlsWithDefault(timePeriod);
  const interval = useLastNonNull(intervalNullable);

  const { isLoading, totalUsage, appDurationsPerPeriod, start, end } =
    useAppDurationsPerPeriod({
      start: interval?.start,
      end: interval?.end,
      period,
    });

  const children = useMemo(
    () => (
      <div className="aspect-video flex-1 mx-1 max-w-full max-h-80">
        <UsageChart
          appDurationsPerPeriod={appDurationsPerPeriod}
          period={period}
          start={start ?? interval.start}
          end={end ?? interval.end}
          xAxisFormatter={xAxisLabelFormatter}
          className="aspect-none"
          maxYIsPeriod
          animationsEnabled={false}
          barRadius={2}
        />
      </div>
    ),
    [appDurationsPerPeriod, period, xAxisLabelFormatter, interval, start, end],
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

import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import { TimePeriodUsageCard } from "@/components/usage-card";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import {
  useAppDurationsPerPeriod,
  useAppSessionUsages,
  useInteractionPeriods,
  useSystemEvents,
} from "@/hooks/use-repo";
import { usePeriodInterval } from "@/hooks/use-today";
import {
  hour24Formatter,
  weekDayFormatter,
  monthDayFormatter,
} from "@/lib/time";
import { DateTime } from "luxon";
import { useState, useMemo } from "react";
import { Gantt } from "@/components/viz/gantt";
import type { Period } from "@/lib/entities";

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
      <div className="h-0 flex-auto overflow-auto [scrollbar-gutter:stable]">
        <div className="flex flex-col gap-4 p-4 overflow-y-auto">
          <div className="grid grid-cols-1 auto-rows-min gap-4 md:grid-cols-3">
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
            <AppUsageBarChartCard
              timePeriod="month"
              period="day"
              xAxisLabelFormatter={monthDayFormatter}
            />
          </div>
          <div className="rounded-xl bg-muted/50 overflow-hidden flex flex-col border border-border">
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
  const startingInterval = usePeriodInterval(timePeriod);
  const [interval, setInterval] = useState(startingInterval);

  const { isLoading, totalUsage, usages, start, end } =
    useAppDurationsPerPeriod({
      start: interval.start,
      end: interval.end,
      period,
    });

  const children = useMemo(
    () => (
      <div className="aspect-video flex-1 mx-1 max-w-full">
        <AppUsageBarChart
          data={usages}
          period={period}
          start={start ?? interval.start}
          end={end ?? interval.end}
          dateTimeFormatter={xAxisLabelFormatter}
          className="aspect-none"
          maxYIsPeriod
          animationsEnabled={false}
          barRadius={2}
        />
      </div>
    ),
    [usages, period, xAxisLabelFormatter, interval, start, end],
  );

  return (
    <TimePeriodUsageCard
      timePeriod={timePeriod}
      interval={interval}
      onIntervalChanged={setInterval}
      children={children}
      isLoading={isLoading}
      totalUsage={totalUsage}
    />
  );
}

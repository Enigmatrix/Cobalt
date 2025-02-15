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
import { useAppDurationsPerPeriod } from "@/hooks/use-repo";
import { type TimePeriod, useTimePeriod } from "@/hooks/use-today";
import {
  durationToTicks,
  dateTimeToTicks,
  hour,
  hour24Formatter,
  day,
  weekDayFormatter,
  monthDayFormatter,
} from "@/lib/time";
import type { Duration, DateTime } from "luxon";
import { useState, useMemo } from "react";

export default function Home() {
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
      <div className="flex flex-1 flex-col gap-4 p-4">
        <div className="grid auto-rows-min gap-4 md:grid-cols-3">
          <AppUsageBarChartCard
            timePeriod="day"
            period={hour}
            xAxisLabelFormatter={hour24Formatter}
          />
          <AppUsageBarChartCard
            timePeriod="week"
            period={day}
            xAxisLabelFormatter={weekDayFormatter}
          />
          <AppUsageBarChartCard
            timePeriod="month"
            period={day}
            xAxisLabelFormatter={monthDayFormatter}
          />
        </div>
        <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min" />
      </div>
    </>
  );
}

function AppUsageBarChartCard({
  timePeriod,
  period,
  xAxisLabelFormatter,
}: {
  timePeriod: TimePeriod;
  period: Duration;
  xAxisLabelFormatter: (dt: DateTime) => string;
}) {
  const startingInterval = useTimePeriod(timePeriod);
  const [interval, setInterval] = useState(startingInterval);

  const { isLoading, totalUsage, usages, start, end } =
    useAppDurationsPerPeriod({
      start: interval.start,
      end: interval.end,
      period: period,
    });

  const children = useMemo(
    () => (
      <div className="aspect-video flex-1 mx-1 max-w-full">
        <AppUsageBarChart
          data={usages}
          periodTicks={durationToTicks(period)}
          rangeMinTicks={dateTimeToTicks(start ?? interval.start)}
          rangeMaxTicks={dateTimeToTicks(end ?? interval.end)}
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

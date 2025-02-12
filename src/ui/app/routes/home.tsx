import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import {
  DayUsageCard,
  MonthUsageCard,
  WeekUsageCard,
  type TimePeriodUsageCardProps,
} from "@/components/usage-card";
import type { DateTime, Duration } from "luxon";
import { useMemo, useState } from "react";
import { useAppDurationsPerPeriod } from "@/hooks/use-repo";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import {
  dateTimeToTicks,
  dayChartPeriod,
  dayXAxisFormatter,
  durationToTicks,
  monthChartPeriod,
  monthXAxisFormatter,
  weekChartPeriod,
  weekXAxisFormatter,
} from "@/lib/time";

function AppUsageBarChartCard({
  card,
  period,
  xAxisLabelFormatter,
}: {
  card: (props: TimePeriodUsageCardProps) => React.ReactNode;
  period: Duration;
  xAxisLabelFormatter: (dt: DateTime) => string;
}) {
  const [range, setRange] = useState<
    { start: DateTime; end: DateTime } | undefined
  >(undefined);

  const { isLoading, totalUsage, usages, start, end } =
    useAppDurationsPerPeriod({
      start: range?.start,
      end: range?.end,
      period: period,
    });

  const children = useMemo(
    () => (
      <div className="aspect-video flex-1 mx-1 max-w-full">
        {!range ? null : (
          <AppUsageBarChart
            data={usages}
            periodTicks={durationToTicks(period)}
            rangeMinTicks={dateTimeToTicks(start ?? range!.start)}
            rangeMaxTicks={dateTimeToTicks(end ?? range!.end)}
            dateTimeFormatter={xAxisLabelFormatter}
            className="aspect-none"
            maxYIsPeriod
            animationsEnabled={false}
          />
        )}
      </div>
    ),
    [usages, period, xAxisLabelFormatter, range, start, end],
  );

  return card({
    usage: totalUsage,
    totalUsage: 0,
    children,
    onChanged: setRange,
    isLoading,
  });
}

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
            card={DayUsageCard}
            period={dayChartPeriod}
            xAxisLabelFormatter={dayXAxisFormatter}
          />
          <AppUsageBarChartCard
            card={WeekUsageCard}
            period={weekChartPeriod}
            xAxisLabelFormatter={weekXAxisFormatter}
          />
          <AppUsageBarChartCard
            card={MonthUsageCard}
            period={monthChartPeriod}
            xAxisLabelFormatter={monthXAxisFormatter}
          />
        </div>
        <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min" />
      </div>
    </>
  );
}

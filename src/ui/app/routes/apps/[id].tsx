import { SidebarTrigger } from "@/components/ui/sidebar";
import type { Route } from "../apps/+types/[id]";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
  BreadcrumbLink,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { useAppState, type EntityMap } from "@/lib/state";
import type { App, Ref, WithGroupedDuration } from "@/lib/entities";
import AppIcon from "@/components/app-icon";
import { DurationText } from "@/components/duration-text";
import { AppUsageBarChart } from "@/components/app-usage-chart";
import { useEffect, useMemo, useState } from "react";
import { getAppDurationsPerPeriod } from "@/lib/repo";
import { Duration, type DateTime } from "luxon";
import { useRefresh } from "@/hooks/use-refresh";
import { dateTimeToTicks, durationToTicks } from "@/lib/time";
import { useToday } from "@/hooks/use-today";

function CardUsage({
  title,
  usage,
  start,
  end,
  period,
  appId,
  formatter,
}: {
  title: string;
  usage: number;
  start: DateTime;
  end: DateTime;
  period: Duration;
  totalUsage: number;
  appId: Ref<App>;
  formatter?: (dateTime: DateTime) => string;
}) {
  const { refreshToken } = useRefresh();
  const [usages, setUsages] = useState<
    EntityMap<App, WithGroupedDuration<App>[]>
  >({});

  useEffect(() => {
    getAppDurationsPerPeriod({
      start,
      end,
      period,
    }).then((usages) => setUsages(usages));
  }, [start, end, period, refreshToken]);

  const data = useMemo(() => ({ [appId]: usages[appId] }), [usages, appId]);

  return (
    <div className="flex flex-col aspect-video rounded-xl bg-muted/50">
      <div className="flex flex-col p-4 pb-1">
        <div className="text-base text-foreground/50">{title}</div>
        <DurationText className="text-xl font-semibold" ticks={usage} />
      </div>

      <AppUsageBarChart
        data={data}
        singleAppId={appId}
        periodTicks={durationToTicks(period)}
        rangeMinTicks={dateTimeToTicks(start)}
        rangeMaxTicks={dateTimeToTicks(end)}
        dateTimeFormatter={formatter}
        gradientBars
        className="aspect-auto h-full flex-1"
        maxYIsPeriod
      />
    </div>
  );
}

export default function App({ params }: Route.ComponentProps) {
  const id = +params.id;
  const app = useAppState((state) => state.apps[id as Ref<App>])!;
  const today = useToday();
  const [dayStart, dayEnd, dayPeriod, dayFormatter] = useMemo(
    () => [
      today.startOf("day"),
      today.endOf("day"),
      Duration.fromObject({ hour: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("HHmm"),
    ],
    [today],
  );
  const [weekStart, weekEnd, weekPeriod, weekFormatter] = useMemo(
    () => [
      today.startOf("week"),
      today.endOf("week"),
      Duration.fromObject({ day: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("EEE"),
    ],
    [today],
  );
  const [monthStart, monthEnd, monthPeriod, monthFormatter] = useMemo(
    () => [
      today.startOf("month"),
      today.endOf("month"),
      Duration.fromObject({ day: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("dd"),
    ],
    [today],
  );

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbLink href="/apps">Apps</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem>
              <BreadcrumbPage className="inline-flex items-center">
                <AppIcon buffer={app.icon} className="w-5 h-5 mr-2" />
                {app.name}
              </BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>
      <div className="flex flex-1 flex-col gap-4 p-4">
        <div className="grid auto-rows-min gap-4 md:grid-cols-3">
          <CardUsage
            title="Today"
            usage={app.usages.usage_today}
            totalUsage={0}
            start={dayStart}
            end={dayEnd}
            period={dayPeriod}
            appId={app.id}
            formatter={dayFormatter}
          />
          <CardUsage
            title="Week"
            usage={app.usages.usage_week}
            totalUsage={0}
            start={weekStart}
            end={weekEnd}
            period={weekPeriod}
            appId={app.id}
            formatter={weekFormatter}
          />
          <CardUsage
            title="Month"
            usage={app.usages.usage_month}
            totalUsage={0}
            start={monthStart}
            end={monthEnd}
            period={monthPeriod}
            appId={app.id}
            formatter={monthFormatter}
          />
        </div>
        <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min" />
      </div>
    </>
  );
}

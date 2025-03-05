import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import { dateTimeToTicks, durationToTicks, type Interval } from "@/lib/time";
import { useMemo, useState } from "react";
import { DateTime, Duration, type DateTimeUnit } from "luxon";
import { Label } from "@/components/ui/label";
import { DateRangePicker } from "@/components/time/date-range-picker";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import { useAppDurationsPerPeriod } from "@/hooks/use-repo";
import {
  Select,
  SelectItem,
  SelectContent,
  SelectValue,
  SelectTrigger,
} from "@/components/ui/select";
import { useTimePeriod } from "@/hooks/use-today";
import {
  VerticalLegend,
  type AppTagId,
} from "@/components/viz/vertical-legend";
import { DurationText } from "@/components/time/duration-text";
import type { App, Ref } from "@/lib/entities";

export default function History() {
  return (
    <div className="flex flex-col h-screen overflow-hidden bg-background">
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-6">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbPage className="text-lg font-medium">
                History
              </BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>
      <div className="flex flex-col flex-1 gap-6 p-6 overflow-hidden">
        <AppUsagePerPeriodHistory />
      </div>
    </div>
  );
}

function AppUsagePerPeriodHistory() {
  const week = useTimePeriod("week");
  const [interval, setInterval] = useState<Interval | null>(week);
  const [period, setPeriod] = useState<DateTimeUnit>("day");
  const periodDuration = useMemo(
    () => Duration.fromObject({ [period]: 1 }),
    [period],
  );
  const {
    totalUsage,
    usages: appUsages,
    start,
    end,
  } = useAppDurationsPerPeriod({
    start: interval?.start,
    end: interval?.end,
    period: periodDuration,
  });

  // State for checked tags/apps
  const [uncheckedApps, setUncheckedApps] = useState<Record<Ref<App>, boolean>>(
    {},
  );
  const [uncheckedTags, setUncheckedTags] = useState<Record<AppTagId, boolean>>(
    {} as Record<AppTagId, boolean>,
  );

  return (
    <div className="flex flex-col flex-1 gap-6 overflow-hidden">
      <div className="flex flex-wrap gap-6 items-center rounded-lg bg-card border border-border p-4 shadow-sm">
        <div className="flex flex-col gap-1.5">
          <Label className="font-medium text-muted-foreground">Usage</Label>
          <DurationText ticks={totalUsage} className="text-lg font-semibold" />
        </div>
        <div className="flex-1 md:min-w-0" />
        <div className="flex flex-col gap-1.5">
          <Label className="font-medium text-muted-foreground">
            Time Range
          </Label>
          <DateRangePicker
            value={interval}
            onChange={setInterval}
            dayGranularity={true}
            className="w-full min-w-32"
          />
        </div>
        <div className="flex flex-col gap-1.5">
          <Label className="font-medium text-muted-foreground">Period</Label>
          <Select
            value={period}
            onValueChange={(s) => setPeriod(s as DateTimeUnit)}
          >
            <SelectTrigger className="min-w-32 font-medium">
              <SelectValue placeholder="Select a period" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="hour">Hour</SelectItem>
              <SelectItem value="day">Day</SelectItem>
              <SelectItem value="week">Week</SelectItem>
              <SelectItem value="month">Month</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      <div className="flex flex-1 min-h-0 overflow-hidden rounded-lg bg-card shadow-sm border border-border">
        <AppUsageBarChart
          data={appUsages}
          periodTicks={durationToTicks(periodDuration)}
          rangeMinTicks={dateTimeToTicks(
            start ?? interval?.start ?? DateTime.now(),
          )}
          rangeMaxTicks={dateTimeToTicks(
            end ?? interval?.end ?? DateTime.now(),
          )}
          className="flex-1 h-full min-w-[400px] p-2"
          maxYIsPeriod={false}
          animationsEnabled={false}
          hideApps={uncheckedApps}
          barRadius={3}
        />

        <VerticalLegend
          className="max-w-[300px] w-[280px] h-full overflow-y-auto p-2"
          uncheckedApps={uncheckedApps}
          setUncheckedApps={setUncheckedApps}
          uncheckedTags={uncheckedTags}
          setUncheckedTags={setUncheckedTags}
        />
      </div>
    </div>
  );
}

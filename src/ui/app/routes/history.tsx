import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import { type Interval } from "@/lib/time";
import { useMemo, useState } from "react";
import { DateTime, Duration } from "luxon";
import { Label } from "@/components/ui/label";
import { DateRangePicker } from "@/components/time/date-range-picker";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import {
  useAppDurationsPerPeriod,
  useInteractionPeriods,
  useSystemEvents,
  useAppSessionUsages,
} from "@/hooks/use-repo";
import {
  Select,
  SelectItem,
  SelectContent,
  SelectValue,
  SelectTrigger,
} from "@/components/ui/select";
import { useTimePeriod } from "@/hooks/use-today";
import { VerticalLegend } from "@/components/viz/vertical-legend";
import { DurationText } from "@/components/time/duration-text";
import type { App, Period, Ref } from "@/lib/entities";
import { Loader2 } from "lucide-react";
import { Gantt } from "@/components/viz/gantt";
import { cn } from "@/lib/utils";

type View = "app-usage" | "session-history";

export default function History() {
  const [view, setView] = useState<View>("app-usage");
  const fullPage = useMemo(() => view === "session-history", [view]);

  return (
    <div className="flex flex-col bg-background h-screen overflow-hidden">
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem>
              <BreadcrumbPage>History</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
        <div className="ml-2">
          <Select value={view} onValueChange={(v) => setView(v as View)}>
            <SelectTrigger className="gap-2">
              <SelectValue placeholder="Select a view" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="app-usage">Grouped Usages</SelectItem>
              <SelectItem value="session-history">Sessions & Usages</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </header>
      <div
        className={cn("flex flex-col flex-1 gap-6 p-4", {
          "overflow-hidden": !fullPage,
          "overflow-y-auto overflow-x-hidden [scrollbar-gutter:stable]":
            fullPage,
        })}
      >
        {view === "app-usage" && <AppUsagePerPeriodHistory />}
        {view === "session-history" && <SessionHistory />}
      </div>
    </div>
  );
}

function AppUsagePerPeriodHistory() {
  const week = useTimePeriod("week");
  const [interval, setInterval] = useState<Interval | null>(week);
  const [period, setPeriod] = useState<Period>("day");

  const {
    isLoading,
    totalUsage,
    usages: appUsages,
    period: loadPeriod,
    start,
    end,
  } = useAppDurationsPerPeriod({
    start: interval?.start,
    end: interval?.end,
    period: period,
  });

  const [yAxisInterval, maxYIsPeriod] = useMemo(() => {
    switch (period) {
      case "hour":
        return [Duration.fromObject({ minutes: 15 }), true];
      case "day":
        return [Duration.fromObject({ hours: 2 }), false];
      case "week":
        return [Duration.fromObject({ hours: 6 }), false];
      case "month":
        return [Duration.fromObject({ days: 1 }), false];
      default:
        throw new Error(`Unknown period: ${period}`);
    }
    // this should take period as a dependency, but we only take in loadPeriod
    // which is a output of useAppDurationsPerPeriod, else we get yaxis flashes
    // with the older data's yaxis interval before the data is loading

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loadPeriod]);

  // State for checked tags/apps
  const [uncheckedApps, setUncheckedApps] = useState<Record<Ref<App>, boolean>>(
    {},
  );

  return (
    <div className="flex flex-col flex-1 gap-6 overflow-hidden">
      <div className="flex flex-wrap gap-6 items-center rounded-lg bg-card border border-border p-4 shadow-xs">
        <div className="flex flex-col gap-1.5">
          <Label className="font-medium text-muted-foreground">Usage</Label>
          <DurationText ticks={totalUsage} className="text-lg font-semibold" />
        </div>
        {isLoading && <Loader2 className="animate-spin w-4 h-4" />}
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
          <Select value={period} onValueChange={(s) => setPeriod(s as Period)}>
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

      <div className="flex flex-1 min-h-0 overflow-hidden rounded-lg bg-card shadow-xs border border-border">
        <AppUsageBarChart
          data={appUsages}
          period={loadPeriod ?? period}
          start={start ?? interval?.start ?? DateTime.now()}
          end={end ?? interval?.end ?? DateTime.now()}
          className="flex-1 h-full min-w-[400px] p-2"
          maxYIsPeriod={maxYIsPeriod}
          interval={yAxisInterval}
          animationsEnabled={false}
          hideApps={uncheckedApps}
          barRadius={3}
        />

        <VerticalLegend
          className="max-w-[300px] w-[280px] h-full overflow-y-auto p-2 border-l [scrollbar-gutter:stable]"
          uncheckedApps={uncheckedApps}
          setUncheckedApps={setUncheckedApps}
        />
      </div>
    </div>
  );
}

function SessionHistory() {
  const week = useTimePeriod("week");
  const [interval, setInterval] = useState<Interval | null>(week);
  const effectiveInterval = interval ?? week;

  const { ret: usages, isLoading: usagesLoading } = useAppSessionUsages({
    start: effectiveInterval.start,
    end: effectiveInterval.end,
  });
  const { ret: interactions, isLoading: interactionPeriodsLoading } =
    useInteractionPeriods({
      start: effectiveInterval.start,
      end: effectiveInterval.end,
    });
  const { ret: systemEvents, isLoading: systemEventsLoading } = useSystemEvents(
    {
      start: effectiveInterval.start,
      end: effectiveInterval.end,
    },
  );

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-wrap gap-6 items-center rounded-lg bg-card border border-border p-4 shadow-xs">
        {/* <div className="flex flex-col gap-1.5">
          <Label className="font-medium text-muted-foreground">Usage</Label>
          <DurationText ticks={totalUsage} className="text-lg font-semibold" />
        </div> */}
        {(usagesLoading ||
          interactionPeriodsLoading ||
          systemEventsLoading) && <Loader2 className="animate-spin w-4 h-4" />}
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
      </div>

      <div className="flex flex-1 min-h-0 rounded-lg bg-card shadow-xs border border-border">
        <div className="max-w-full">
          <Gantt
            usages={usages}
            usagesLoading={usagesLoading}
            interactionPeriods={interactions}
            interactionPeriodsLoading={interactionPeriodsLoading}
            systemEvents={systemEvents}
            systemEventsLoading={systemEventsLoading}
            rangeStart={effectiveInterval.start}
            rangeEnd={effectiveInterval.end}
          />
        </div>
      </div>
    </div>
  );
}

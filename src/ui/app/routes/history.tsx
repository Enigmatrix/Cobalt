import { DateRangePicker } from "@/components/time/date-range-picker";
import { DurationText } from "@/components/time/duration-text";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import { FormItem } from "@/components/ui/form";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { Gantt } from "@/components/viz/gantt2";
import { UsageChart, type GroupBy } from "@/components/viz/usage-chart";
import { VerticalLegend } from "@/components/viz/vertical-legend";
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
import type { App, Ref } from "@/lib/entities";
import type { Period } from "@/lib/time";
import { cn } from "@/lib/utils";
import { ChevronLeftIcon, ChevronRightIcon, Loader2 } from "lucide-react";
import { DateTime, Duration } from "luxon";
import { createContext, useContext, useMemo, useState } from "react";

type View = "app-usage" | "session-history";

type HistoryContextType = ReturnType<typeof useIntervalControlsWithDefault>;
const HistoryContext = createContext<HistoryContextType | null>(null);

function useHistoryContext() {
  const context = useContext(HistoryContext);
  if (!context) {
    throw new Error("useHistoryContext must be used within a HistoryProvider");
  }
  return context;
}

export default function History() {
  const [view, setView] = useState<View>("app-usage");
  const fullPage = useMemo(() => view === "session-history", [view]);
  const { interval, setInterval, canGoNext, goNext, canGoPrev, goPrev } =
    useIntervalControlsWithDefault("week");

  const historyContextValue = useMemo(
    () => ({
      interval,
      setInterval,
      canGoNext,
      goNext,
      canGoPrev,
      goPrev,
    }),
    [interval, setInterval, canGoNext, goNext, canGoPrev, goPrev],
  );

  return (
    <HistoryContext.Provider value={historyContextValue}>
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
                <SelectItem value="session-history">
                  Sessions & Usages
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex-1" />

          <div className="flex gap-1 items-center">
            <Button
              variant="ghost"
              size="icon"
              disabled={!canGoPrev}
              onClick={goPrev}
            >
              <ChevronLeftIcon className="size-4" />
            </Button>
            <FormItem>
              <DateRangePicker
                value={interval}
                onChange={setInterval}
                dayGranularity={true}
                className="w-full min-w-32"
              />
            </FormItem>
            <Button
              variant="ghost"
              size="icon"
              disabled={!canGoNext}
              onClick={goNext}
            >
              <ChevronRightIcon className="size-4" />
            </Button>
          </div>
        </header>
        <div
          className={cn("flex flex-col flex-1", {
            "overflow-hidden": !fullPage,
            "overflow-y-auto overflow-x-hidden [scrollbar-gutter:stable]":
              fullPage,
          })}
        >
          <div className="flex flex-col flex-1 gap-6 p-4">
            {view === "app-usage" && <AppUsagePerPeriodHistory />}
            {view === "session-history" && <SessionHistory />}
          </div>
        </div>
      </div>
    </HistoryContext.Provider>
  );
}

function AppUsagePerPeriodHistory() {
  const [period, setPeriod] = useState<Period>("day");
  const { interval } = useHistoryContext();

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

  const [groupBy, setGroupBy] = useState<GroupBy>("app");

  return (
    <div className="flex flex-col flex-1 gap-6 overflow-hidden">
      <div className="flex flex-wrap gap-6 items-center rounded-lg bg-card border border-border p-4 shadow-xs">
        <div className="flex flex-col gap-1.5">
          <Label className="font-medium text-muted-foreground">Usage</Label>
          <DurationText ticks={totalUsage} className="text-lg font-semibold" />
        </div>
        {isLoading && <Loader2 className="animate-spin w-4 h-4" />}
        <div className="flex-1 md:min-w-0" />
        <FormItem>
          <Label className="font-medium text-muted-foreground place-self-end">
            View
          </Label>
          <Select
            value={groupBy}
            onValueChange={(s) => setGroupBy(s as GroupBy)}
          >
            <SelectTrigger className="min-w-32 font-medium">
              <SelectValue placeholder="Select a view" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={"app" as GroupBy}>Apps</SelectItem>
              <SelectItem value={"tag" as GroupBy}>Tags</SelectItem>
              <SelectItem value={"tag-show-untagged" as GroupBy}>
                Tags (untagged seperate)
              </SelectItem>
            </SelectContent>
          </Select>
        </FormItem>
        <FormItem>
          <Label className="font-medium text-muted-foreground place-self-end">
            Period
          </Label>
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
        </FormItem>
      </div>

      <div className="flex flex-1 min-h-0 overflow-hidden rounded-lg bg-card shadow-xs border border-border">
        <UsageChart
          usages={appUsages}
          period={loadPeriod ?? period}
          start={start ?? interval?.start ?? DateTime.now()}
          end={end ?? interval?.end ?? DateTime.now()}
          className="flex-1 h-full min-w-[400px] p-2"
          maxYIsPeriod={maxYIsPeriod}
          yAxisInterval={yAxisInterval}
          animationsEnabled={false}
          hiddenApps={uncheckedApps}
          groupBy={groupBy}
          barRadius={3}
        />

        <VerticalLegend
          className="max-w-[300px] w-[280px] h-full overflow-y-auto p-2 border-l"
          uncheckedApps={uncheckedApps}
          setUncheckedApps={setUncheckedApps}
        />
      </div>
    </div>
  );
}

function SessionHistory() {
  const week = usePeriodInterval("week");
  const { interval } = useHistoryContext();

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
    <div className="sticky rounded-lg bg-card shadow-xs border border-border overflow-clip">
      <Gantt
        usages={usages}
        usagesLoading={usagesLoading}
        interactionPeriods={interactions}
        interactionPeriodsLoading={interactionPeriodsLoading}
        systemEvents={systemEvents}
        systemEventsLoading={systemEventsLoading}
        interval={effectiveInterval}
      />
    </div>
  );
}

import { SidebarTrigger } from "@/components/ui/sidebar";
import type { Route } from "../apps/+types/[id]";
import { Separator } from "@/components/ui/separator";
import { useDebouncedCallback } from "use-debounce";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
  BreadcrumbLink,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { useAppState } from "@/lib/state";
import { type App, type Period, type Ref, type Tag } from "@/lib/entities";
import AppIcon from "@/components/app/app-icon";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import { useCallback, useMemo, useState } from "react";
import { DateTime } from "luxon";
import { useApp, useTag } from "@/hooks/use-refresh";
import {
  dateTimeToTicks,
  hour24Formatter,
  monthDayFormatter,
  ticksToDateTime,
  ticksToDuration,
  weekDayFormatter,
} from "@/lib/time";
import { Text } from "@/components/ui/text";
import { Button } from "@/components/ui/button";
import { Check, ChevronsUpDown, Copy } from "lucide-react";
import { useClipboard } from "@/hooks/use-clipboard";
import { EditableText } from "@/components/editable-text";
import { ColorPicker } from "@/components/color-picker";
import _ from "lodash";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { NavLink } from "react-router";
import { TimePeriodUsageCard } from "@/components/usage-card";
import Heatmap from "@/components/viz/heatmap";
import {
  useAppDurationsPerPeriod,
  useAppSessionUsages,
} from "@/hooks/use-repo";
import { useTimePeriod } from "@/hooks/use-today";
import { Gantt } from "@/components/viz/gantt";
import { ChooseTag } from "@/components/tag/choose-tag";
import type { TimePeriod } from "@/lib/time";

export default function App({ params }: Route.ComponentProps) {
  const id = +params.id;
  const app = useApp(id as Ref<App>)!;
  const updateApp = useAppState((state) => state.updateApp);
  const [color, setColorInner] = useState(app.color);
  const debouncedUpdateColor = useDebouncedCallback(async (color: string) => {
    await updateApp({ ...app, color });
  }, 500);

  const setColor = useCallback(
    async (color: string) => {
      setColorInner(color);
      await debouncedUpdateColor(color);
    },
    [setColorInner, debouncedUpdateColor],
  );

  const { copy, hasCopied } = useClipboard();

  const yearPeriod = useTimePeriod("year");
  const [yearInterval, setYearInterval] = useState(yearPeriod);

  const {
    isLoading: isYearDataLoading,
    appUsage: yearUsage,
    totalUsage: yearTotalUsage,
    usages: yearUsages,
    start: yearRangeStart,
  } = useAppDurationsPerPeriod({
    start: yearInterval.start,
    end: yearInterval.end,
    period: "day",
    appId: app.id,
  });
  const yearData = useMemo(() => {
    return new Map(
      _(yearUsages[app.id] || [])
        .map(
          (appDur) =>
            [+ticksToDateTime(appDur.group), appDur.duration] as const,
        )
        .value(),
    );
  }, [yearUsages, app.id]);

  const scaling = useCallback((value: number) => {
    return _.clamp(ticksToDuration(value).rescale().hours / 8, 0.2, 1);
  }, []);

  const dayRange = useTimePeriod("day");
  const { ret: appSessionUsages, isLoading: appSessionUsagesLoading } =
    useAppSessionUsages({
      start: dayRange.start,
      end: dayRange.end,
    });
  const onlyAppSessionUsages = useMemo(() => {
    return appSessionUsages[app.id]
      ? { [app.id]: appSessionUsages[app.id] }
      : {};
  }, [appSessionUsages, app.id]);

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
                <Text>{app.name}</Text>
              </BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>

      <div className="h-0 flex-auto overflow-auto [scrollbar-gutter:stable]">
        <div className="flex flex-col gap-4 p-4">
          {/* App Info */}
          <div className="rounded-xl bg-card border border-border p-6">
            <div className="flex flex-col gap-4">
              {/* Header with name and icon */}
              <div className="flex items-center gap-4">
                <AppIcon buffer={app.icon} className="w-12 h-12 shrink-0" />
                <div className="min-w-0 shrink flex flex-col">
                  <div className="min-w-0 flex gap-4">
                    <EditableText
                      text={app.name}
                      className="min-w-0 text-2xl font-semibold grow-0"
                      buttonClassName="ml-1"
                      onSubmit={async (v) =>
                        await updateApp({ ...app, name: v })
                      }
                    />
                    <TagSelect
                      tagId={app.tag_id}
                      setTagId={async (tagId) =>
                        await updateApp({ ...app, tag_id: tagId })
                      }
                      className="min-w-0"
                    />
                  </div>
                  <Text className="text-muted-foreground">{app.company}</Text>
                </div>
                <div className="flex-1" />
                <ColorPicker
                  className="min-w-0 w-fit"
                  color={color}
                  onChange={setColor}
                />
              </div>

              {/* Description */}
              <EditableText
                className="text-muted-foreground self-start"
                buttonClassName="text-muted-foreground/50"
                text={app.description}
                onSubmit={async (v) =>
                  await updateApp({ ...app, description: v })
                }
              />

              {/* App Identity */}
              <div className="text-sm inline-flex border-border border rounded-lg overflow-hidden max-w-fit min-w-0 bg-muted/30 items-center">
                <div className="bg-muted px-3 py-1.5 border-r border-border font-medium">
                  {app.identity.tag === "Uwp" ? "UWP" : "Win32"}
                </div>

                <Text className="font-mono pl-3 pr-1 py-1.5 text-muted-foreground">
                  {app.identity.tag === "Uwp"
                    ? app.identity.aumid
                    : app.identity.path}
                </Text>
                <Button
                  variant="ghost"
                  className="h-auto p-2 rounded-none rounded-r-lg text-muted-foreground"
                  onClick={() =>
                    copy(
                      app.identity.tag === "Uwp"
                        ? app.identity.aumid
                        : app.identity.path,
                    )
                  }
                >
                  {hasCopied ? <Check /> : <Copy />}
                </Button>
              </div>
            </div>
          </div>

          <div className="grid grid-cols-1 auto-rows-min gap-4 md:grid-cols-3">
            <AppUsageBarChartCard
              timePeriod="day"
              period="hour"
              xAxisLabelFormatter={hour24Formatter}
              appId={app.id}
            />
            <AppUsageBarChartCard
              timePeriod="week"
              period="day"
              xAxisLabelFormatter={weekDayFormatter}
              appId={app.id}
            />
            <AppUsageBarChartCard
              timePeriod="month"
              period="day"
              xAxisLabelFormatter={monthDayFormatter}
              appId={app.id}
            />
          </div>
          <TimePeriodUsageCard
            timePeriod="year"
            usage={yearUsage}
            totalUsage={yearTotalUsage}
            interval={yearInterval}
            onIntervalChanged={setYearInterval}
            isLoading={isYearDataLoading}
          >
            <div className="p-4">
              <Heatmap
                data={yearData}
                scaling={scaling}
                startDate={yearRangeStart ?? yearInterval.start}
                fullCellColorRgb={app.color}
                innerClassName="min-h-[200px]"
                firstDayOfMonthClassName="stroke-card-foreground/50"
                appId={app.id}
              />
            </div>
          </TimePeriodUsageCard>

          <div className="rounded-xl bg-muted/50 overflow-hidden flex flex-col border border-border">
            <Gantt
              usages={onlyAppSessionUsages}
              usagesLoading={appSessionUsagesLoading}
              defaultExpanded={{ [app.id]: true }}
              rangeStart={dayRange.start}
              rangeEnd={dayRange.end}
            />
          </div>
        </div>
      </div>
    </>
  );
}

function TagSelect({
  tagId,
  setTagId,
  className,
}: {
  tagId: Ref<Tag> | null;
  setTagId: (tagId: Ref<Tag> | null) => Promise<void>;
  className?: ClassValue;
}) {
  const tag = useTag(tagId);

  return tag ? (
    <Badge
      variant="outline"
      style={{
        borderColor: tag.color,
        color: tag.color,
        backgroundColor: "rgba(255, 255, 255, 0.2)",
      }}
      className={cn("whitespace-nowrap", className)}
    >
      <NavLink to={`/tags/${tagId}`} className="min-w-0">
        <Text className="max-w-32">{tag.name}</Text>
      </NavLink>
      <ChooseTag
        value={tagId}
        onValueChanged={setTagId}
        render={() => (
          <Button
            size="icon"
            variant="ghost"
            className="p-1 ml-2 w-auto h-auto"
          >
            <ChevronsUpDown />
          </Button>
        )}
      />
    </Badge>
  ) : (
    <Badge
      variant="outline"
      className={cn("whitespace-nowrap text-muted-foreground", className)}
    >
      <Text className="max-w-32">Untagged</Text>
      <ChooseTag
        value={tagId}
        onValueChanged={setTagId}
        render={() => (
          <Button
            size="icon"
            variant="ghost"
            className="p-1 ml-2 w-auto h-auto"
          >
            <ChevronsUpDown />
          </Button>
        )}
      />
    </Badge>
  );
}

function AppUsageBarChartCard({
  timePeriod,
  period,
  xAxisLabelFormatter,
  appId,
}: {
  timePeriod: TimePeriod;
  period: Period;
  xAxisLabelFormatter: (dt: DateTime) => string;
  appId: Ref<App>;
}) {
  const startingInterval = useTimePeriod(timePeriod);
  const [interval, setInterval] = useState(startingInterval);

  const { isLoading, appUsage, totalUsage, usages, start, end } =
    useAppDurationsPerPeriod({
      start: interval.start,
      end: interval.end,
      period,
      appId,
    });

  const children = useMemo(
    () => (
      <div className="aspect-video flex-1 mx-1 max-w-full">
        <AppUsageBarChart
          data={usages}
          singleAppId={appId}
          rangeMinTicks={dateTimeToTicks(start ?? interval.start)}
          rangeMaxTicks={dateTimeToTicks(end ?? interval.end)}
          dateTimeFormatter={xAxisLabelFormatter}
          period={period}
          gradientBars
          className="aspect-none"
          maxYIsPeriod
        />
      </div>
    ),
    [usages, period, xAxisLabelFormatter, interval, start, end, appId],
  );

  return (
    <TimePeriodUsageCard
      timePeriod={timePeriod}
      interval={interval}
      onIntervalChanged={setInterval}
      children={children}
      isLoading={isLoading}
      usage={appUsage}
      totalUsage={totalUsage}
    />
  );
}

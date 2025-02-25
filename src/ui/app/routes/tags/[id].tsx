import { SidebarTrigger } from "@/components/ui/sidebar";
import type { Route } from "../tags/+types/[id]";
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
import { type App, type Ref, type Tag } from "@/lib/entities";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import { useCallback, useMemo, useState } from "react";
import { DateTime, Duration } from "luxon";
import { useApps, useTag } from "@/hooks/use-refresh";
import {
  dateTimeToTicks,
  day,
  durationToTicks,
  hour,
  hour24Formatter,
  monthDayFormatter,
  ticksToDateTime,
  ticksToDuration,
  weekDayFormatter,
} from "@/lib/time";
import { Text } from "@/components/ui/text";
import { PlusIcon, TagIcon, TrashIcon } from "lucide-react";
import { EditableText } from "@/components/editable-text";
import { ColorPicker } from "@/components/color-picker";
import _ from "lodash";
import { TimePeriodUsageCard } from "@/components/usage-card";
import Heatmap from "@/components/viz/heatmap";
import {
  useAppDurationsPerPeriod,
  useAppSessionUsages,
  useTagDurationsPerPeriod,
} from "@/hooks/use-repo";
import { Button, buttonVariants } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { useNavigate } from "react-router";
import AppIcon from "@/components/app/app-icon";
import { useAppsSearch } from "@/hooks/use-search";
import { MultiSelect } from "@/components/multi-select";
import { AppBadge } from "@/components/app/app-list-item";
import { useTimePeriod, type TimePeriod } from "@/hooks/use-today";
import { Gantt } from "@/components/viz/gantt";

export default function Tag({ params }: Route.ComponentProps) {
  const id = +params.id;
  const tag = useTag(id as Ref<Tag>)!;
  const apps = useApps();
  const [search, setSearch, appsFiltered] = useAppsSearch(apps);
  const appListIds = useMemo(
    () => appsFiltered.map((app) => app.id),
    [appsFiltered],
  );
  const allAppOptions = useMemo(
    () =>
      apps.map((app) => ({
        id: app.id,
        label: app.name,
        icon: ({ className }: { className?: string }) => (
          <AppIcon buffer={app.icon} className={className} />
        ),
        render: ({ remove }: { remove: () => void }) => (
          <AppBadge app={app} remove={remove} />
        ),
      })),
    [apps],
  );
  const updateTag = useAppState((state) => state.updateTag);
  const removeTag = useAppState((state) => state.removeTag);
  const updateTagApps = useAppState((state) => state.updateTagApps);

  const [color, setColorInner] = useState(tag.color);
  const debouncedUpdateColor = useDebouncedCallback(async (color: string) => {
    await updateTag({ ...tag, color });
  }, 500);

  const setColor = useCallback(
    async (color: string) => {
      setColorInner(color);
      await debouncedUpdateColor(color);
    },
    [setColorInner, debouncedUpdateColor],
  );

  const navigate = useNavigate();
  const remove = useCallback(async () => {
    await navigate("/tags");
    await removeTag(tag.id);
  }, [removeTag, navigate, tag.id]);

  const yearPeriod = useTimePeriod("year");
  const [yearInterval, setYearInterval] = useState(yearPeriod);

  const {
    isLoading: isYearDataLoading,
    tagUsage: yearUsage,
    totalUsage: yearTotalUsage,
    usages: yearUsages,
    start: yearRangeStart,
  } = useTagDurationsPerPeriod({
    start: yearInterval.start,
    end: yearInterval.end,
    period: day,
    tag,
  });
  const yearData = useMemo(() => {
    return new Map(
      _(yearUsages[tag.id] || [])
        .map(
          (appDur) =>
            [+ticksToDateTime(appDur.group), appDur.duration] as const,
        )
        .value(),
    );
  }, [yearUsages, tag.id]);

  const scaling = useCallback((value: number) => {
    return _.clamp(ticksToDuration(value).rescale().hours / 8, 0.2, 1);
  }, []);

  const setTagApps = useCallback(
    async (apps: Ref<App>[]) => {
      await updateTagApps(tag, apps);
    },
    [tag, updateTagApps],
  );

  const dayRange = useTimePeriod("day");

  const { ret: appSessionUsages, isLoading: appSessionUsagesLoading } =
    useAppSessionUsages({
      start: dayRange.start,
      end: dayRange.end,
    });
  const tagAppSessionUsages = useMemo(() => {
    return _(tag.apps)
      .filter((appId) => appSessionUsages[appId] !== undefined)
      .map((appId) => [appId, appSessionUsages[appId]] as const)
      .fromPairs()
      .value();
  }, [appSessionUsages, tag]);

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbLink href="/tags">Tags</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem>
              <BreadcrumbPage className="inline-flex items-center">
                <TagIcon
                  className="w-5 h-5 mr-2"
                  style={{ color: tag.color }}
                />
                <Text>{tag.name}</Text>
              </BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>

      <div className="h-0 flex-auto overflow-auto [scrollbar-gutter:stable]">
        <div className="flex flex-col gap-4 p-4">
          {/* App Info */}
          <div className="rounded-xl bg-card border border-border px-6 pt-6 pb-4">
            <div className="flex flex-col gap-6">
              {/* Header with name and icon */}
              <div className="flex items-center gap-4">
                <TagIcon
                  className="w-12 h-12 shrink-0"
                  style={{ color: tag.color }}
                />
                <div className="min-w-0 shrink flex flex-col">
                  <div className="min-w-0 flex gap-4">
                    <EditableText
                      text={tag.name}
                      className="min-w-0 text-2xl font-semibold grow-0"
                      buttonClassName="ml-1"
                      onSubmit={async (v) =>
                        await updateTag({ ...tag, name: v })
                      }
                    />
                  </div>
                </div>
                <div className="flex-1" />
                <ColorPicker
                  className="min-w-0 w-fit"
                  color={color}
                  onChange={setColor}
                />
                <AlertDialog>
                  <AlertDialogTrigger asChild>
                    <Button size="icon" variant="outline">
                      <TrashIcon />
                    </Button>
                  </AlertDialogTrigger>
                  <AlertDialogContent>
                    <AlertDialogHeader>
                      <AlertDialogTitle>Remove Tag?</AlertDialogTitle>
                      <AlertDialogDescription>
                        This action cannot be undone. All Apps using this Tag
                        will be marked as Untagged.
                      </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                      <AlertDialogCancel>Cancel</AlertDialogCancel>
                      <AlertDialogAction
                        onClick={remove}
                        className={buttonVariants({ variant: "destructive" })}
                      >
                        Remove
                      </AlertDialogAction>
                    </AlertDialogFooter>
                  </AlertDialogContent>
                </AlertDialog>
              </div>

              <MultiSelect
                className="min-h-14 border-none w-fit"
                options={allAppOptions}
                onValueChange={setTagApps}
                defaultValue={tag.apps}
                placeholder="No Apps. Add some!"
                rightIcon={(className) => <PlusIcon className={className} />}
                maxCount={1000}
                search={search}
                onSearchChanged={setSearch}
                filteredValues={appListIds}
              />
            </div>
          </div>

          <div className="grid grid-cols-1 auto-rows-min gap-4 md:grid-cols-3">
            <TagUsageBarChartCard
              timePeriod="day"
              period={hour}
              xAxisLabelFormatter={hour24Formatter}
              tag={tag}
            />
            <TagUsageBarChartCard
              timePeriod="week"
              period={day}
              xAxisLabelFormatter={weekDayFormatter}
              tag={tag}
            />
            <TagUsageBarChartCard
              timePeriod="month"
              period={day}
              xAxisLabelFormatter={monthDayFormatter}
              tag={tag}
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
                fullCellColorRgb={tag.color}
                innerClassName="min-h-[200px]"
                firstDayOfMonthClassName="stroke-card-foreground/50"
                tagId={tag.id}
              />
            </div>
          </TimePeriodUsageCard>

          <div className="rounded-xl bg-muted/50 overflow-hidden flex flex-col border border-border">
            <Gantt
              usages={tagAppSessionUsages}
              usagesLoading={appSessionUsagesLoading}
              rangeStart={dayRange.start}
              rangeEnd={dayRange.end}
            />
          </div>
        </div>
      </div>
    </>
  );
}

function TagUsageBarChartCard({
  timePeriod,
  period,
  xAxisLabelFormatter,
  tag,
}: {
  timePeriod: TimePeriod;
  period: Duration;
  xAxisLabelFormatter: (dt: DateTime) => string;
  tag: Tag;
}) {
  const startingInterval = useTimePeriod(timePeriod);
  const [interval, setInterval] = useState(startingInterval);

  const {
    isLoading,
    totalUsage,
    usages: appUsages,
    start,
    end,
  } = useAppDurationsPerPeriod({
    start: interval.start,
    end: interval.end,
    period: period,
  });
  const { usages, tagUsage } = useMemo(() => {
    const usages = _(tag.apps)
      .map((appId) => [appId, appUsages[appId]])
      .fromPairs()
      .value();
    const tagUsage = _(usages).values().flatten().sumBy("duration") ?? 0;
    return { usages, tagUsage };
  }, [appUsages, tag]);

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
      usage={tagUsage}
      totalUsage={totalUsage}
    />
  );
}

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
import { TagIcon, TrashIcon, XIcon } from "lucide-react";
import { EditableText } from "@/components/editable-text";
import { ColorPicker } from "@/components/color-picker";
import _ from "lodash";
import {
  DayUsageCard,
  MonthUsageCard,
  WeekUsageCard,
  YearUsageCard,
  type TimePeriodUsageCardProps,
} from "@/components/usage-card";
import Heatmap from "@/components/viz/heatmap";
import {
  useAppDurationsPerPeriod,
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
import { Badge } from "@/components/ui/badge";
import AppIcon from "@/components/app/app-icon";
import { useSearch } from "@/hooks/use-search";
import { MultiSelect } from "@/components/multi-select";

function TagUsageBarChartCard({
  card,
  period,
  xAxisLabelFormatter,
  tag,
}: {
  card: (props: TimePeriodUsageCardProps) => React.ReactNode;
  period: Duration;
  xAxisLabelFormatter: (dt: DateTime) => string;
  tag: Tag;
}) {
  const [range, setRange] = useState<
    { start: DateTime; end: DateTime } | undefined
  >(undefined);

  const {
    isLoading,
    totalUsage,
    usages: appUsages,
    start,
    end,
  } = useAppDurationsPerPeriod({
    start: range?.start,
    end: range?.end,
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
        {!range ? null : (
          <AppUsageBarChart
            data={usages}
            periodTicks={durationToTicks(period)}
            rangeMinTicks={dateTimeToTicks(start ?? range!.start)}
            rangeMaxTicks={dateTimeToTicks(end ?? range!.end)}
            dateTimeFormatter={xAxisLabelFormatter}
            className="aspect-none"
            maxYIsPeriod
          />
        )}
      </div>
    ),
    [usages, period, xAxisLabelFormatter, range, start, end],
  );

  return card({
    usage: tagUsage,
    totalUsage: totalUsage,
    children,
    onChanged: setRange,
    isLoading,
  });
}

function AppBadge({ app, remove }: { app: App; remove: () => void }) {
  return (
    <Badge
      className="whitespace-nowrap min-w-0 m-1 font-normal border-border border rounded-md h-8 text-foreground"
      style={{
        backgroundColor: "rgba(255, 255, 255, 0.1)",
      }}
    >
      <AppIcon buffer={app.icon} className="h-5 w-5 mr-2" />
      <Text className="text-base">{app.name}</Text>
      <XIcon
        className="ml-2 h-4 w-4 cursor-pointer"
        onClick={(event) => {
          event.stopPropagation();
          remove();
        }}
      />
    </Badge>
  );
}

export default function Tag({ params }: Route.ComponentProps) {
  const id = +params.id;
  const tag = useTag(id as Ref<Tag>)!;
  const apps = useApps();
  const [search, setSearch, appsFiltered] = useSearch(apps, [
    "name",
    "company",
    "description",
  ]);
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

  const [range, setRange] = useState<
    { start: DateTime; end: DateTime } | undefined
  >(undefined);

  const {
    isLoading: isYearDataLoading,
    tagUsage: yearUsage,
    totalUsage: yearTotalUsage,
    usages: yearUsages,
    start: yearRangeStart,
  } = useTagDurationsPerPeriod({
    start: range?.start,
    end: range?.end,
    period: day,
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
      <div className="flex flex-1 flex-col gap-4 p-4">
        {/* App Info */}
        <div className="rounded-xl bg-muted/50 px-6 pt-6 pb-4">
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
                    onSubmit={async (v) => await updateTag({ ...tag, name: v })}
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
                      This action cannot be undone. All Apps using this Tag will
                      be marked as Untagged.
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
              className="min-h-14"
              options={allAppOptions}
              onValueChange={setTagApps}
              defaultValue={tag.apps}
              placeholder="Select Apps"
              maxCount={1000}
              search={search}
              onSearchChanged={setSearch}
              filteredValues={appListIds}
            />
          </div>
        </div>

        <div className="grid auto-rows-min gap-4 md:grid-cols-3">
          <TagUsageBarChartCard
            card={DayUsageCard}
            period={hour}
            xAxisLabelFormatter={hour24Formatter}
            tag={tag}
          />
          <TagUsageBarChartCard
            card={WeekUsageCard}
            period={day}
            xAxisLabelFormatter={weekDayFormatter}
            tag={tag}
          />
          <TagUsageBarChartCard
            card={MonthUsageCard}
            period={day}
            xAxisLabelFormatter={monthDayFormatter}
            tag={tag}
          />
        </div>
        <YearUsageCard
          usage={yearUsage}
          totalUsage={yearTotalUsage}
          onChanged={setRange}
          isLoading={isYearDataLoading}
        >
          <div className="p-4">
            {!range?.start ? (
              // avoid CLS
              <div className="min-h-[200px]" />
            ) : (
              <Heatmap
                data={yearData}
                scaling={scaling}
                startDate={yearRangeStart ?? range.start}
                fullCellColorRgb={tag.color}
                innerClassName="min-h-[200px]"
                firstDayOfMonthClassName="stroke-card-foreground/50"
              />
            )}
          </div>
        </YearUsageCard>
      </div>
    </>
  );
}

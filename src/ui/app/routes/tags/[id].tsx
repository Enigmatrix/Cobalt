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
import { type Ref, type Tag } from "@/lib/entities";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import { useCallback, useMemo, useState } from "react";
import { DateTime, Duration } from "luxon";
import { useTag } from "@/hooks/use-refresh";
import {
  dateTimeToTicks,
  durationToTicks,
  ticksToDateTime,
  ticksToDuration,
} from "@/lib/time";
import { Text } from "@/components/ui/text";
import { TagIcon } from "lucide-react";
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

const dayChartPeriod = Duration.fromObject({ hour: 1 });
const weekChartPeriod = Duration.fromObject({ day: 1 });
const monthChartPeriod = Duration.fromObject({ day: 1 });
const yearChartPeriod = Duration.fromObject({ day: 1 });
const dayXAxisFormatter = (dateTime: DateTime) => dateTime.toFormat("HHmm");
const weekXAxisFormatter = (dateTime: DateTime) => dateTime.toFormat("EEE");
const monthXAxisFormatter = (dateTime: DateTime) => dateTime.toFormat("dd");

export default function Tag({ params }: Route.ComponentProps) {
  const id = +params.id;
  const tag = useTag(id as Ref<Tag>)!;
  const updateTag = useAppState((state) => state.updateTag);
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
    period: yearChartPeriod,
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
        <div className="rounded-xl bg-muted/50 p-6">
          <div className="flex flex-col gap-4">
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
            </div>
          </div>
        </div>

        <div className="grid auto-rows-min gap-4 md:grid-cols-3">
          <TagUsageBarChartCard
            card={DayUsageCard}
            period={dayChartPeriod}
            xAxisLabelFormatter={dayXAxisFormatter}
            tag={tag}
          />
          <TagUsageBarChartCard
            card={WeekUsageCard}
            period={weekChartPeriod}
            xAxisLabelFormatter={weekXAxisFormatter}
            tag={tag}
          />
          <TagUsageBarChartCard
            card={MonthUsageCard}
            period={monthChartPeriod}
            xAxisLabelFormatter={monthXAxisFormatter}
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

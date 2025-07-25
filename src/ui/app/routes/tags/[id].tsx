import { ChooseMultiApps } from "@/components/app/choose-multi-apps";
import { ColorPicker } from "@/components/color-picker";
import { EditableText } from "@/components/editable-text";
import { ScoreBadge, ScoreEdit } from "@/components/tag/score";
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
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button, buttonVariants } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { Text } from "@/components/ui/text";
import { NextButton, PrevButton, UsageCard } from "@/components/usage-card";
import { Gantt } from "@/components/viz/gantt2";
import Heatmap from "@/components/viz/heatmap";
import { UsageChart } from "@/components/viz/usage-chart";
import { useLastNonNull } from "@/hooks/use-last";
import { useAlerts, useTag } from "@/hooks/use-refresh";
import {
  useAppDurationsPerPeriod,
  useAppSessionUsages,
  useTagDurationsPerPeriod,
} from "@/hooks/use-repo";
import {
  useIntervalControlsWithDefault,
  usePeriodInterval,
} from "@/hooks/use-time";
import type { App, Ref, Tag } from "@/lib/entities";
import { useAppState } from "@/lib/state";
import {
  hour24Formatter,
  monthDayFormatter,
  ticksToDateTime,
  ticksToDuration,
  weekDayFormatter,
  type Period,
} from "@/lib/time";
import _ from "lodash";
import { TagIcon, TrashIcon } from "lucide-react";
import { DateTime } from "luxon";
import { useCallback, useMemo, useState } from "react";
import { NavLink, useNavigate } from "react-router";
import { useDebouncedCallback } from "use-debounce";
import type { Route } from "../tags/+types/[id]";

export default function Page({ params }: Route.ComponentProps) {
  const id = +params.id as Ref<Tag>;
  const tag = useTag(id);
  if (!tag) return null;
  return <TagPage tag={tag} />;
}

function TagPage({ tag }: { tag: Tag }) {
  const updateTag = useAppState((state) => state.updateTag);
  const removeTag = useAppState((state) => state.removeTag);
  const updateTagApps = useAppState((state) => state.updateTagApps);

  const [color, setColorInner] = useState(tag.color);
  const [score, setScoreInner] = useState(tag.score);
  const debouncedUpdateColor = useDebouncedCallback(async (color: string) => {
    await updateTag({ ...tag, color });
  }, 500);

  const debouncedUpdateScore = useDebouncedCallback(async (score: number) => {
    await updateTag({ ...tag, score });
  }, 500);

  const setColor = useCallback(
    async (color: string) => {
      setColorInner(color);
      await debouncedUpdateColor(color);
    },
    [setColorInner, debouncedUpdateColor],
  );

  const setScore = useCallback(
    async (score: number) => {
      setScoreInner(score);
      await debouncedUpdateScore(score);
    },
    [setScoreInner, debouncedUpdateScore],
  );

  const navigate = useNavigate();
  const remove = useCallback(async () => {
    await navigate("/tags");
    await removeTag(tag.id);
  }, [removeTag, navigate, tag.id]);

  const {
    interval: yearIntervalNullable,
    canGoNext: yearCanGoNext,
    goNext: yearGoNext,
    canGoPrev: yearCanGoPrev,
    goPrev: yearGoPrev,
  } = useIntervalControlsWithDefault("year");
  const yearInterval = useLastNonNull(yearIntervalNullable);

  const alerts = useAlerts();
  const tagAlerts = useMemo(
    () =>
      alerts.filter(
        (alert) => alert.target.tag === "tag" && alert.target.id === tag.id,
      ),
    [alerts, tag],
  );

  const {
    isLoading: isYearDataLoading,
    tagUsage: yearUsage,
    totalUsage: yearTotalUsage,
    usages: yearUsages,
    start: yearStart,
  } = useTagDurationsPerPeriod({
    start: yearInterval.start,
    end: yearInterval.end,
    period: "day",
    tag,
  });
  const yearData = useMemo(() => {
    return new Map(
      _(yearUsages[tag.id] ?? [])
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

  const day = usePeriodInterval("day");

  const { ret: appSessionUsages, isLoading: appSessionUsagesLoading } =
    useAppSessionUsages({
      start: day.start,
      end: day.end,
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
        <Breadcrumb className="overflow-hidden">
          <BreadcrumbList className="flex-nowrap overflow-hidden">
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbLink asChild>
                <NavLink to="/tags">Tags</NavLink>
              </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem className="overflow-hidden">
              <BreadcrumbPage className="inline-flex items-center overflow-hidden">
                <TagIcon
                  className="w-5 h-5 mr-2 shrink-0"
                  style={{ color: tag.color }}
                />
                <Text>{tag.name}</Text>
              </BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>

      <div className="h-0 flex-auto overflow-y-auto overflow-x-hidden [scrollbar-gutter:stable]">
        <div className="flex flex-col gap-4 p-4">
          {/* Tag Info */}
          <div className="rounded-xl bg-card border border-border px-6 pt-6 pb-4">
            <div className="flex flex-col gap-6">
              {/* Header with name and icon */}
              <div className="flex items-center gap-4">
                <TagIcon
                  className="w-12 h-12 shrink-0"
                  style={{ color: tag.color }}
                />
                <div className="min-w-0 shrink flex flex-col gap-2">
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
                  <ScoreEdit
                    score={score}
                    onScoreChange={setScore}
                    className="self-start min-w-0"
                  >
                    <ScoreBadge
                      className="text-sm py-0.5 pl-3 pr-2 min-w-0 rounded-md"
                      score={score}
                    />
                  </ScoreEdit>
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
                        This action cannot be undone.
                        {tag.apps.length > 0 &&
                          ` ${tag.apps.length} App${tag.apps.length === 1 ? "" : "s"} using this Tag will be marked as Untagged.`}
                        {tagAlerts.length > 0 &&
                          ` ${tagAlerts.length} Alert${tagAlerts.length === 1 ? "" : "s"} using this Tag (and their Reminders and history) will be removed.`}
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

              <ChooseMultiApps value={tag.apps} onValueChanged={setTagApps} />
            </div>
          </div>

          <div className="grid auto-rows-min gap-4 grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
            <TagUsageBarChartCard
              timePeriod="day"
              period="hour"
              xAxisLabelFormatter={hour24Formatter}
              tag={tag}
            />
            <TagUsageBarChartCard
              timePeriod="week"
              period="day"
              xAxisLabelFormatter={weekDayFormatter}
              tag={tag}
            />
          </div>

          <div className="grid auto-rows-min gap-4 grid-cols-1">
            <TagUsageBarChartCard
              timePeriod="month"
              period="day"
              xAxisLabelFormatter={monthDayFormatter}
              tag={tag}
            />
          </div>

          <UsageCard
            usage={yearUsage}
            totalUsage={yearTotalUsage}
            interval={yearInterval}
            actions={
              <>
                <PrevButton
                  canGoPrev={yearCanGoPrev}
                  isLoading={isYearDataLoading}
                  goPrev={yearGoPrev}
                />
                <NextButton
                  canGoNext={yearCanGoNext}
                  isLoading={isYearDataLoading}
                  goNext={yearGoNext}
                />
              </>
            }
          >
            <div className="p-4">
              <Heatmap
                data={yearData}
                scaling={scaling}
                startDate={yearStart ?? yearInterval.start}
                fullCellColorRgb={tag.color}
                innerClassName="min-h-[200px]"
                firstDayOfMonthClassName="stroke-card-foreground/50"
                tagId={tag.id}
              />
            </div>
          </UsageCard>

          <div className="sticky rounded-xl bg-muted/50 border border-border overflow-clip">
            <Gantt
              usages={tagAppSessionUsages}
              usagesLoading={appSessionUsagesLoading}
              interval={day}
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
  timePeriod: Period;
  period: Period;
  xAxisLabelFormatter: (dt: DateTime) => string;
  tag: Tag;
}) {
  const {
    interval: intervalNullable,
    canGoNext,
    goNext,
    canGoPrev,
    goPrev,
  } = useIntervalControlsWithDefault(timePeriod);
  const interval = useLastNonNull(intervalNullable);

  const { isLoading, totalUsage, appDurationsPerPeriod, start, end } =
    useAppDurationsPerPeriod({
      start: interval.start,
      end: interval.end,
      period,
    });
  const { totalTagUsage } = useMemo(() => {
    const totalTagUsage = _(tag.apps)
      .flatMap((appId) => appDurationsPerPeriod[appId] ?? [])
      .sumBy("duration");
    return { totalTagUsage };
  }, [appDurationsPerPeriod, tag]);

  const children = useMemo(
    () => (
      <div className="aspect-video flex-1 mx-1 max-w-full max-h-80">
        <UsageChart
          appDurationsPerPeriod={appDurationsPerPeriod}
          onlyShowOneTag={tag.id}
          period={period}
          start={start ?? interval.start}
          end={end ?? interval.end}
          xAxisFormatter={xAxisLabelFormatter}
          className="aspect-none"
          maxYIsPeriod
          barRadius={2}
        />
      </div>
    ),
    [
      appDurationsPerPeriod,
      period,
      xAxisLabelFormatter,
      interval,
      start,
      end,
      tag.id,
    ],
  );

  return (
    <UsageCard
      interval={interval}
      usage={totalTagUsage}
      totalUsage={totalUsage}
      children={children}
      actions={
        <>
          <PrevButton
            canGoPrev={canGoPrev}
            isLoading={isLoading}
            goPrev={goPrev}
          />
          <NextButton
            canGoNext={canGoNext}
            isLoading={isLoading}
            goNext={goNext}
          />
        </>
      }
    />
  );
}

import AppIcon from "@/components/app/app-icon";
import { ColorPicker } from "@/components/color-picker";
import { EditableText } from "@/components/editable-text";
import { ChooseTag } from "@/components/tag/choose-tag";
import { ScoreCircle } from "@/components/tag/score";
import { Badge } from "@/components/ui/badge";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { Text } from "@/components/ui/text";
import { NextButton, PrevButton, UsageCard } from "@/components/usage-card";
import { Gantt } from "@/components/viz/gantt2";
import Heatmap from "@/components/viz/heatmap";
import { UsageChart } from "@/components/viz/usage-chart";
import { useClipboard } from "@/hooks/use-clipboard";
import { useApp, useTag } from "@/hooks/use-refresh";
import {
  useAppDurationsPerPeriod,
  useAppSessionUsages,
  useSingleEntityUsageFromPerPeriod,
  useTotalUsageFromPerPeriod,
} from "@/hooks/use-repo";
import {
  useIntervalControlsWithDefault,
  usePeriodInterval,
} from "@/hooks/use-time";
import { type App, type Ref, type Tag } from "@/lib/entities";
import { useAppState } from "@/lib/state";
import {
  hour24Formatter,
  monthDayFormatter,
  ticksToDateTime,
  ticksToDuration,
  weekDayFormatter,
  type Period,
} from "@/lib/time";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import _ from "lodash";
import { Check, ChevronsUpDown, Copy } from "lucide-react";
import { DateTime } from "luxon";
import { useCallback, useMemo, useState } from "react";
import { NavLink } from "react-router";
import { useDebouncedCallback } from "use-debounce";
import type { Route } from "../apps/+types/[id]";

export default function Page({ params }: Route.ComponentProps) {
  const id = +params.id as Ref<App>;
  const app = useApp(id);
  if (!app) return null;
  return <AppPage app={app} />;
}

function AppPage({ app }: { app: App }) {
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

  const {
    interval: yearInterval,
    canGoNext: yearCanGoNext,
    goNext: yearGoNext,
    canGoPrev: yearCanGoPrev,
    goPrev: yearGoPrev,
  } = useIntervalControlsWithDefault("year");

  const { isLoading: isYearDataLoading, ret: yearUsages } =
    useAppDurationsPerPeriod({
      ...yearInterval,
      period: "day",
    });
  const yearTotalUsage = useTotalUsageFromPerPeriod(yearUsages);
  const yearUsage = useSingleEntityUsageFromPerPeriod(yearUsages, app.id);

  const yearData = useMemo(() => {
    return new Map(
      _(yearUsages[app.id] ?? [])
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

  const day = usePeriodInterval("day");
  const { ret: appSessionUsages, isLoading: appSessionUsagesLoading } =
    useAppSessionUsages(day);
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
        <Breadcrumb className="overflow-hidden">
          <BreadcrumbList className="flex-nowrap overflow-hidden">
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbLink asChild>
                <NavLink to="/apps">Apps</NavLink>
              </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem className="overflow-hidden">
              <BreadcrumbPage className="inline-flex items-center overflow-hidden">
                <AppIcon appIcon={app.icon} className="w-5 h-5 mr-2 shrink-0" />
                <Text>{app.name}</Text>
              </BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>

      <div className="h-0 flex-auto overflow-y-auto overflow-x-hidden [scrollbar-gutter:stable]">
        <div className="flex flex-col gap-4 p-4">
          {/* App Info */}
          <div className="rounded-xl bg-card border border-border p-6">
            <div className="flex flex-col gap-4">
              {/* Header with name and icon */}
              <div className="flex items-center gap-4">
                <AppIcon appIcon={app.icon} className="w-12 h-12 shrink-0" />
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
                      tagId={app.tagId}
                      setTagId={async (tagId) =>
                        await updateApp({ ...app, tagId: tagId })
                      }
                      className="min-w-0"
                    />
                  </div>
                  {app.identity.tag !== "website" && (
                    <Text className="text-muted-foreground">{app.company}</Text>
                  )}
                </div>
                <div className="flex-1" />
                <ColorPicker
                  className="min-w-0 w-fit"
                  color={color}
                  onChange={setColor}
                />
              </div>

              {/* Description */}
              <div className="flex">
                <EditableText
                  className="text-muted-foreground max-w-full"
                  buttonClassName="text-muted-foreground/50"
                  text={app.description}
                  onSubmit={async (v) =>
                    await updateApp({ ...app, description: v })
                  }
                />
              </div>

              {/* App Identity */}
              <div className="text-sm inline-flex border-border border rounded-lg overflow-hidden max-w-fit min-w-0 bg-muted/30 items-center">
                <div className="bg-muted px-3 py-1.5 border-r border-border font-medium">
                  {app.identity.tag === "uwp"
                    ? "UWP"
                    : app.identity.tag === "win32"
                      ? "Win32"
                      : "Web"}
                </div>

                <Text className="font-mono pl-3 pr-1 py-1.5 text-muted-foreground">
                  {app.identity.tag === "uwp"
                    ? app.identity.aumid
                    : app.identity.tag === "win32"
                      ? app.identity.path
                      : app.identity.baseUrl}
                </Text>
                <Button
                  variant="ghost"
                  className="h-auto p-2 rounded-none rounded-r-lg text-muted-foreground"
                  onClick={() =>
                    copy(
                      app.identity.tag === "uwp"
                        ? app.identity.aumid
                        : app.identity.tag === "win32"
                          ? app.identity.path
                          : app.identity.baseUrl,
                    )
                  }
                >
                  {hasCopied ? <Check /> : <Copy />}
                </Button>
              </div>
            </div>
          </div>

          <div className="grid auto-rows-min gap-4 grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
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
          </div>

          <div className="grid auto-rows-min gap-4 grid-cols-1">
            <AppUsageBarChartCard
              timePeriod="month"
              period="day"
              xAxisLabelFormatter={monthDayFormatter}
              appId={app.id}
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
                startDate={yearInterval.start}
                fullCellColorRgb={app.color}
                innerClassName="min-h-[200px]"
                firstDayOfMonthClassName="stroke-card-foreground/50"
                appId={app.id}
              />
            </div>
          </UsageCard>

          <div className="sticky rounded-xl bg-muted/50 border border-border overflow-clip">
            <Gantt
              usages={onlyAppSessionUsages}
              usagesLoading={appSessionUsagesLoading}
              defaultExpanded={{ [app.id]: true }}
              interval={day}
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
      <NavLink to={`/tags/${tagId}`} className="min-w-0 flex items-center">
        <Text className="max-w-32 ml-1">{tag.name}</Text>
        <ScoreCircle score={tag.score} className="ml-2" />
      </NavLink>
      <ChooseTag
        value={tagId}
        onValueChanged={setTagId}
        render={() => (
          <Button
            size="icon"
            variant="ghost"
            className="p-1 ml-1 w-auto h-auto"
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
      <Text className="max-w-32 ml-1">Untagged</Text>
      <ChooseTag
        value={tagId}
        onValueChanged={setTagId}
        render={() => (
          <Button
            size="icon"
            variant="ghost"
            className="p-1 ml-1 w-auto h-auto"
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
  timePeriod: Period;
  period: Period;
  xAxisLabelFormatter: (dt: DateTime) => string;
  appId: Ref<App>;
}) {
  const { interval, canGoNext, goNext, canGoPrev, goPrev } =
    useIntervalControlsWithDefault(timePeriod);

  const { isLoading, ret: appDurationsPerPeriod } = useAppDurationsPerPeriod({
    ...interval,
    period,
  });

  const totalUsage = useTotalUsageFromPerPeriod(appDurationsPerPeriod);
  const totalAppUsage = useSingleEntityUsageFromPerPeriod(
    appDurationsPerPeriod,
    appId,
  );

  const children = useMemo(
    () => (
      <div className="aspect-video flex-1 mx-1 max-w-full max-h-80">
        <UsageChart
          appDurationsPerPeriod={appDurationsPerPeriod}
          onlyShowOneApp={appId}
          start={interval.start}
          end={interval.end}
          xAxisFormatter={xAxisLabelFormatter}
          period={period}
          gradientBars
          className="aspect-none"
          maxYIsPeriod
        />
      </div>
    ),
    [appDurationsPerPeriod, period, xAxisLabelFormatter, interval, appId],
  );

  return (
    <UsageCard
      interval={interval}
      usage={totalAppUsage}
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

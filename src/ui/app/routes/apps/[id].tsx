import { SidebarTrigger } from "@/components/ui/sidebar";
import type { Route } from "../apps/+types/[id]";
import { Separator } from "@/components/ui/separator";
import { useThrottledCallback } from "use-debounce";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
  BreadcrumbLink,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { useAppState, type EntityMap } from "@/lib/state";
import {
  isUwp,
  type App,
  type Ref,
  type WithGroupedDuration,
} from "@/lib/entities";
import AppIcon from "@/components/app-icon";
import { DurationText } from "@/components/duration-text";
import { AppUsageBarChart } from "@/components/app-usage-chart";
import { useCallback, useEffect, useMemo, useState } from "react";
import { getAppDurationsPerPeriod } from "@/lib/repo";
import { Duration, type DateTime } from "luxon";
import { useRefresh } from "@/hooks/use-refresh";
import { dateTimeToTicks, durationToTicks } from "@/lib/time";
import { useToday } from "@/hooks/use-today";
import { MiniTagItem } from ".";
import { Text } from "@/components/ui/text";
import { Button } from "@/components/ui/button";
import { Check, ChevronLeft, ChevronRight, Copy } from "lucide-react";
import { useClipboard } from "@/hooks/use-clipboard";
import { EditableText } from "@/components/editable-text";
import { ColorPicker } from "@/components/color-picker";
import _ from "lodash";

function CardUsage({
  titleFormatter,
  start: originalStart,
  end: originalEnd,
  next,
  prev,
  period,
  appId,
  chartDateFormatter,
}: {
  titleFormatter: (dt: DateTime) => string;
  start: DateTime;
  end: DateTime;
  next: (start: DateTime, end: DateTime) => { start: DateTime; end: DateTime };
  prev: (start: DateTime, end: DateTime) => { start: DateTime; end: DateTime };
  period: Duration;
  appId: Ref<App>;
  chartDateFormatter?: (dateTime: DateTime) => string;
}) {
  const { refreshToken } = useRefresh();
  const [usages, setUsages] = useState<
    EntityMap<App, WithGroupedDuration<App>[]>
  >({});
  const [totalUsage, setTotalUsage] = useState<number>(0);
  const [usage, setUsage] = useState<number>(0);
  const [start, setStart] = useState(originalStart);
  const [end, setEnd] = useState(originalEnd);

  useEffect(() => {
    getAppDurationsPerPeriod({
      start,
      end,
      period,
    }).then((usages) => {
      setUsages(usages);
      setUsage(_(usages[appId]).sumBy("duration"));
      setTotalUsage(_(usages).values().flatten().sumBy("duration"));
    });
  }, [start, end, period, setUsage, setTotalUsage, setUsages, refreshToken]);

  const nextFn = useCallback(() => {
    const { start: nextStart, end: nextEnd } = next(start, end);
    setStart(nextStart);
    setEnd(nextEnd);
  }, [next, start, end, setStart, setEnd]);

  const prevFn = useCallback(() => {
    const { start: prevStart, end: prevEnd } = prev(start, end);
    setStart(prevStart);
    setEnd(prevEnd);
  }, [prev, start, end, setStart, setEnd]);

  const data = useMemo(() => ({ [appId]: usages[appId] }), [usages, appId]);

  return (
    <div className="flex flex-col aspect-video rounded-xl bg-muted/50">
      <div className="flex items-center">
        <div className="flex flex-col p-4 pb-1">
          <div className="text-base text-foreground/50">
            {titleFormatter(start)}
          </div>
          <DurationText className="text-xl font-semibold" ticks={usage} />
        </div>

        <div className="flex-1" />

        <div className="flex m-2">
          <Button variant="ghost" size="icon" className="" onClick={prevFn}>
            <ChevronLeft />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className=""
            onClick={nextFn}
            disabled={+end === +originalEnd}
          >
            <ChevronRight />
          </Button>
        </div>
      </div>

      <AppUsageBarChart
        data={data}
        singleAppId={appId}
        periodTicks={durationToTicks(period)}
        rangeMinTicks={dateTimeToTicks(start)}
        rangeMaxTicks={dateTimeToTicks(end)}
        dateTimeFormatter={chartDateFormatter}
        gradientBars
        className="aspect-auto h-full flex-1 mx-1"
        maxYIsPeriod
      />
    </div>
  );
}

export default function App({ params }: Route.ComponentProps) {
  const id = +params.id;
  const app = useAppState((state) => state.apps[id as Ref<App>])!;
  const updateApp = useAppState((state) => state.updateApp);

  const allTags = useAppState((state) => state.tags);
  const { handleStaleTags } = useRefresh();
  const tag = useMemo(() => {
    if (app.tag_id === null) {
      return null;
    }
    return handleStaleTags([allTags[app.tag_id]])[0] ?? null;
  }, [allTags, handleStaleTags, app.tag_id]);

  // TODO change title
  const today = useToday();
  const [
    dayStart,
    dayEnd,
    dayPeriod,
    dayFormatter,
    dayNext,
    dayPrev,
    dayTitleFormatter,
  ] = useMemo(
    () => [
      today.startOf("day"),
      today.endOf("day"),
      Duration.fromObject({ hour: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("HHmm"),
      (start: DateTime, end: DateTime) => ({
        start: start.plus({ day: 1 }),
        end: end.plus({ day: 1 }),
      }),
      (start: DateTime, end: DateTime) => ({
        start: start.minus({ day: 1 }),
        end: end.minus({ day: 1 }),
      }),
      (dt: DateTime) => {
        if (+today.startOf("day") === +dt) return "Today";
        if (+today.startOf("day").minus({ day: 1 }) === +dt) return "Yesterday";
        const format = dt.year === today.year ? "dd LLL" : "dd LLL yyyy";
        return dt.toFormat(format);
      },
    ],
    [today],
  );

  const [
    weekStart,
    weekEnd,
    weekPeriod,
    weekFormatter,
    weekNext,
    weekPrev,
    weekTitleFormatter,
  ] = useMemo(
    () => [
      today.startOf("week"),
      today.endOf("week"),
      Duration.fromObject({ day: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("EEE"),
      (start: DateTime, end: DateTime) => ({
        start: start.plus({ week: 1 }),
        end: end.plus({ week: 1 }),
      }),
      (start: DateTime, end: DateTime) => ({
        start: start.minus({ week: 1 }),
        end: end.minus({ week: 1 }),
      }),
      (dt: DateTime) => {
        if (+today.startOf("week") === +dt) return "This Week";
        if (+today.startOf("week").minus({ week: 1 }) === +dt)
          return "Last Week";
        const format =
          dt.year === today.year && dt.endOf("week").year === today.year
            ? "dd LLL"
            : "dd LLL yyyy";
        return dt.toFormat(format) + " - " + dt.endOf("week").toFormat(format);
      },
    ],
    [today],
  );
  const [
    monthStart,
    monthEnd,
    monthPeriod,
    monthFormatter,
    monthNext,
    monthPrev,
    monthTitleFormatter,
  ] = useMemo(
    () => [
      today.startOf("month"),
      today.plus({ month: 1 }).startOf("month"),
      Duration.fromObject({ day: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("dd"),
      (start: DateTime) => ({
        start: start.plus({ month: 1 }).startOf("month"),
        end: start
          .plus({ month: 1 })
          .startOf("month")
          .plus({ month: 1 })
          .startOf("month"),
      }),
      (start: DateTime) => ({
        start: start.minus({ month: 1 }).startOf("month"),
        end: start
          .minus({ month: 1 })
          .startOf("month")
          .plus({ month: 1 })
          .startOf("month"),
      }),
      (dt: DateTime) => {
        if (+today.startOf("month") === +dt) return "This Month";
        if (+today.startOf("month").minus({ month: 1 }) === +dt)
          return "Last Month";
        const format = dt.year === today.year ? "LLL" : "LLL yyyy";
        return dt.toFormat(format);
      },
    ],
    [today],
  );
  const [color, setColorInner] = useState(app.color);

  const debouncedUpdateColor = useThrottledCallback(async (color: string) => {
    await updateApp({ ...app, color });
  }, 500);

  const setColor = useCallback(
    async (color: string) => {
      setColorInner(color);
      await debouncedUpdateColor(color);
    },
    [updateApp, setColorInner],
  );

  const { copy, hasCopied } = useClipboard();

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
      <div className="flex flex-1 flex-col gap-4 p-4">
        {/* App Info */}
        <div className="rounded-xl bg-muted/50 p-6">
          <div className="flex flex-col gap-4">
            {/* Header with name and icon */}
            <div className="flex items-center gap-4">
              <AppIcon buffer={app.icon} className="w-12 h-12 shrink-0" />
              <div className="min-w-0 shrink flex flex-col">
                <div className="flex gap-4">
                  <EditableText
                    text={app.name}
                    className="text-2xl font-semibold grow-0"
                    buttonClassName="ml-1"
                    onSubmit={async (v) => await updateApp({ ...app, name: v })}
                  />
                  {tag && <MiniTagItem tag={tag} />}
                </div>
                <Text className="text-muted-foreground">{app.company}</Text>
              </div>
              <div className="flex-1" />
              <ColorPicker
                className="w-fit"
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
                {isUwp(app.identity) ? "UWP" : "Win32"}
              </div>

              <Text className="font-mono pl-3 pr-1 py-1.5 text-muted-foreground">
                {isUwp(app.identity)
                  ? app.identity.Uwp.aumid
                  : app.identity.Win32.path}
              </Text>
              <Button
                variant="ghost"
                className="h-auto p-2 rounded-none rounded-r-lg text-muted-foreground"
                onClick={() =>
                  copy(
                    isUwp(app.identity)
                      ? app.identity.Uwp.aumid
                      : app.identity.Win32.path,
                  )
                }
              >
                {hasCopied ? <Check /> : <Copy />}
              </Button>
            </div>
          </div>
        </div>

        <div className="grid auto-rows-min gap-4 md:grid-cols-3">
          <CardUsage
            start={dayStart}
            end={dayEnd}
            period={dayPeriod}
            appId={app.id}
            chartDateFormatter={dayFormatter}
            titleFormatter={dayTitleFormatter}
            next={dayNext}
            prev={dayPrev}
          />
          <CardUsage
            start={weekStart}
            end={weekEnd}
            period={weekPeriod}
            appId={app.id}
            chartDateFormatter={weekFormatter}
            titleFormatter={weekTitleFormatter}
            next={weekNext}
            prev={weekPrev}
          />
          <CardUsage
            start={monthStart}
            end={monthEnd}
            period={monthPeriod}
            appId={app.id}
            chartDateFormatter={monthFormatter}
            titleFormatter={monthTitleFormatter}
            next={monthNext}
            prev={monthPrev}
          />
        </div>
        <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min" />
      </div>
    </>
  );
}

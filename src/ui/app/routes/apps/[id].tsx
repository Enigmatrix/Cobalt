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
  type Tag,
  type WithGroupedDuration,
} from "@/lib/entities";
import AppIcon from "@/components/app-icon";
import { DurationText } from "@/components/duration-text";
import { AppUsageBarChart } from "@/components/app-usage-chart";
import { useCallback, useEffect, useMemo, useState } from "react";
import { getAppDurationsPerPeriod } from "@/lib/repo";
import { Duration, type DateTime } from "luxon";
import { useApp, useRefresh, useTag, useTags } from "@/hooks/use-refresh";
import { dateTimeToTicks, durationToTicks, ticksToDateTime } from "@/lib/time";
import { useToday } from "@/hooks/use-today";
import { Text } from "@/components/ui/text";
import { Button } from "@/components/ui/button";
import {
  Check,
  ChevronLeft,
  ChevronRight,
  ChevronsUpDown,
  Copy,
  Plus,
  TagIcon,
} from "lucide-react";
import { useClipboard } from "@/hooks/use-clipboard";
import { EditableText } from "@/components/editable-text";
import { ColorPicker } from "@/components/color-picker";
import _ from "lodash";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { NavLink } from "react-router";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { SearchBar } from "@/components/search-bar";
import { useSearch } from "@/hooks/use-search";
import { CreateTagDialog } from "@/components/create-tag-dialog";
import Heatmap from "@/components/heatmap";

function ChooseTagPopover({
  tagId,
  setTagId: setTagIdInner,
}: {
  tagId: Ref<Tag> | null;
  setTagId: (tagId: Ref<Tag> | null) => Promise<void>;
}) {
  const createTag = useAppState((state) => state.createTag);
  const tags = useTags();
  const [query, setQuery, filteredTags] = useSearch(tags, ["name"]);
  const [open, setOpen] = useState(false);

  const setTagId = useCallback(
    async (tagId: Ref<Tag> | null) => {
      await setTagIdInner(tagId);
      setOpen(false);
    },
    [setTagIdInner, setOpen],
  );

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button size="icon" variant="ghost" className="p-1 ml-2 w-auto h-auto">
          <ChevronsUpDown />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[20rem] z-10">
        {tags.length !== 0 && (
          <SearchBar value={query} onChange={(e) => setQuery(e.target.value)} />
        )}
        <div className="flex flex-col my-4 gap-1 overflow-y-auto max-h-[20rem] px-2">
          {tags.length === 0 ? (
            <div className="m-auto flex flex-col text-muted-foreground py-8 gap-2 items-center">
              <TagIcon className="w-10 h-10 m-4" />
              <p className="text-lg">No tags exist. Create some!</p>
            </div>
          ) : (
            filteredTags.map((tag) => {
              return (
                <div
                  key={tag.id}
                  className={cn(
                    "flex cursor-pointer hover:bg-muted p-2 rounded-md gap-2",
                    {
                      "bg-muted/80": tag.id === tagId,
                    },
                  )}
                  onClick={async () => await setTagId(tag.id)}
                >
                  <TagIcon style={{ color: tag.color }} />
                  <Text className="text-foreground">{tag.name}</Text>

                  {tag.id === tagId && (
                    <Check className="ml-auto text-muted-foreground/50" />
                  )}
                </div>
              );
            })
          )}
        </div>
        <div className="flex gap-2">
          <div className="flex-1"></div>
          <Button
            variant="outline"
            className={cn({
              hidden: tagId === null,
            })}
            onClick={async () => await setTagId(null)}
          >
            Clear
          </Button>
          <CreateTagDialog
            trigger={
              <Button variant={tags.length === 0 ? "default" : "outline"}>
                <Plus />
                Create Tag
              </Button>
            }
            onSubmit={async (tag) => {
              const tagId = await createTag(tag);
              await setTagId(tagId);
            }}
          ></CreateTagDialog>
        </div>
      </PopoverContent>
    </Popover>
  );
}

function ChooseTag({
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
      <ChooseTagPopover tagId={tagId} setTagId={setTagId} />
    </Badge>
  ) : (
    <Badge
      variant="outline"
      className={cn("whitespace-nowrap text-muted-foreground", className)}
    >
      <Text className="max-w-32">Untagged</Text>
      <ChooseTagPopover tagId={tagId} setTagId={setTagId} />
    </Badge>
  );
}

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
    <div className="flex flex-col rounded-xl bg-muted/50">
      <div className="flex items-center">
        <div className="flex flex-col p-4 pb-1 min-w-0">
          <div className="whitespace-nowrap text-base text-foreground/50">
            {titleFormatter(start)}
          </div>
          <div className="flex gap-2 items-baseline font-semibold">
            <DurationText className="text-xl" ticks={usage} />
            {usage !== 0 && (
              <>
                <span className="text-xl text-muted-foreground">/</span>
                <DurationText
                  className="text-muted-foreground"
                  ticks={totalUsage}
                />
              </>
            )}
          </div>
        </div>

        <div className="flex-1" />

        {/* hide button controls between md: and lg: */}
        <div className="flex m-2 max-lg:hidden max-md:block">
          <Button variant="ghost" size="icon" onClick={prevFn}>
            <ChevronLeft />
          </Button>
          <Button
            variant="ghost"
            size="icon"
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
        className="aspect-video flex-1 mx-1 max-w-full"
        maxYIsPeriod
      />
    </div>
  );
}

export default function App({ params }: Route.ComponentProps) {
  const id = +params.id;
  const app = useApp(id as Ref<App>)!;
  const updateApp = useAppState((state) => state.updateApp);

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

  const [yearStart, yearEnd] = useMemo(
    () => [today.startOf("year"), today.endOf("year")],
    [],
  );
  const [yearData, setYearData] = useState(new Map());

  useEffect(() => {
    getAppDurationsPerPeriod({
      start: yearStart,
      end: yearEnd,
      period: Duration.fromObject({ day: 1 }),
    }).then((usages) => {
      setYearData(
        new Map(
          _(usages[app.id])
            .map(
              (appDur) =>
                [+ticksToDateTime(appDur.group), appDur.duration] as const,
            )
            .value(),
        ),
      );
    });
  }, [yearStart, yearEnd, app]);

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
                <div className="min-w-0 flex gap-4">
                  <EditableText
                    text={app.name}
                    className="min-w-0 text-2xl font-semibold grow-0"
                    buttonClassName="ml-1"
                    onSubmit={async (v) => await updateApp({ ...app, name: v })}
                  />
                  <ChooseTag
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
        <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min p-4">
          <Heatmap
            axisClassName="fill-muted-foreground"
            data={yearData}
            startDate={yearStart}
          />
        </div>
      </div>
    </>
  );
}

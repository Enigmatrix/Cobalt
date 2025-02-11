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
import { AppUsageBarChart } from "@/components/app-usage-chart";
import { useCallback, useMemo, useState } from "react";
import { getAppDurationsPerPeriod } from "@/lib/repo";
import { Duration, type DateTime } from "luxon";
import { useApp, useTag, useTags } from "@/hooks/use-refresh";
import { dateTimeToTicks, durationToTicks } from "@/lib/time";
import { Text } from "@/components/ui/text";
import { Button } from "@/components/ui/button";
import { Check, ChevronsUpDown, Copy, Plus, TagIcon } from "lucide-react";
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
import {
  DayUsageCard,
  MonthUsageCard,
  WeekUsageCard,
  type TimePeriodUsageCardProps,
} from "@/components/usage-card";

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

function AppUsageBarChartCard({
  card,
  period,
  xAxisLabelFormatter,
  appId,
}: {
  card: (props: TimePeriodUsageCardProps) => React.ReactNode;
  period: Duration;
  xAxisLabelFormatter: (dt: DateTime) => string;
  appId: Ref<App>;
}) {
  const [data, setData] = useState<{
    usage: number;
    totalUsage: number;
    data: EntityMap<App, WithGroupedDuration<App>[]>;
    start: DateTime;
    end: DateTime;
  } | null>(null);

  const setStartEnd = useCallback(
    function (start: DateTime, end: DateTime) {
      getAppDurationsPerPeriod({ start, end, period }).then((usages) => {
        const usage = _(usages[appId]).sumBy("duration");
        const totalUsage = _(usages).values().flatten().sumBy("duration");
        const data = { [appId]: usages[appId] };
        setData({
          usage,
          totalUsage,
          data,
          start,
          end,
        });
      });
    },
    [setData, period, appId],
  );

  const children = useMemo(() => {
    return !data ? null : (
      <AppUsageBarChart
        data={data.data}
        singleAppId={appId}
        periodTicks={durationToTicks(period)}
        rangeMinTicks={dateTimeToTicks(data.start)}
        rangeMaxTicks={dateTimeToTicks(data.end)}
        dateTimeFormatter={xAxisLabelFormatter}
        gradientBars
        className="aspect-video flex-1 mx-1 max-w-full"
        maxYIsPeriod
      />
    );
  }, [data, appId]);

  return card({
    usage: data?.usage || 0,
    totalUsage: data?.totalUsage || 0,
    children,
    onChanged: setStartEnd,
  });
}

export default function App({ params }: Route.ComponentProps) {
  const id = +params.id;
  const app = useApp(id as Ref<App>)!;
  const updateApp = useAppState((state) => state.updateApp);

  const [dayPeriod, dayFormatter] = useMemo(
    () => [
      Duration.fromObject({ hour: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("HHmm"),
    ],
    [],
  );

  const [weekPeriod, weekFormatter] = useMemo(
    () => [
      Duration.fromObject({ day: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("EEE"),
    ],
    [],
  );
  const [monthPeriod, monthFormatter] = useMemo(
    () => [
      Duration.fromObject({ day: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("dd"),
    ],
    [],
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
          <AppUsageBarChartCard
            card={DayUsageCard}
            period={dayPeriod}
            xAxisLabelFormatter={dayFormatter}
            appId={app.id}
          />
          <AppUsageBarChartCard
            card={WeekUsageCard}
            period={weekPeriod}
            xAxisLabelFormatter={weekFormatter}
            appId={app.id}
          />
          <AppUsageBarChartCard
            card={MonthUsageCard}
            period={monthPeriod}
            xAxisLabelFormatter={monthFormatter}
            appId={app.id}
          />
        </div>
        <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min" />
      </div>
    </>
  );
}

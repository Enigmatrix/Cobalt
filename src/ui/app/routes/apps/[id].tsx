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
import { HorizontalOverflowList } from "@/components/overflow-list";
import { MiniTagItem } from ".";
import _ from "lodash";
import { Badge } from "@/components/ui/badge";
import { Text } from "@/components/ui/text";
import { Button } from "@/components/ui/button";
import { Check, Copy } from "lucide-react";
import { useClipboard } from "@/hooks/use-clipboard";
import { EditableText } from "@/components/editable-text";
import { ColorPicker } from "@/components/color-picker";

function CardUsage({
  title,
  usage,
  start,
  end,
  period,
  appId,
  formatter,
}: {
  title: string;
  usage: number;
  start: DateTime;
  end: DateTime;
  period: Duration;
  totalUsage: number;
  appId: Ref<App>;
  formatter?: (dateTime: DateTime) => string;
}) {
  const { refreshToken } = useRefresh();
  const [usages, setUsages] = useState<
    EntityMap<App, WithGroupedDuration<App>[]>
  >({});

  useEffect(() => {
    getAppDurationsPerPeriod({
      start,
      end,
      period,
    }).then((usages) => setUsages(usages));
  }, [start, end, period, refreshToken]);

  const data = useMemo(() => ({ [appId]: usages[appId] }), [usages, appId]);

  return (
    <div className="flex flex-col aspect-video rounded-xl bg-muted/50">
      <div className="flex flex-col p-4 pb-1">
        <div className="text-base text-foreground/50">{title}</div>
        <DurationText className="text-xl font-semibold" ticks={usage} />
      </div>

      <AppUsageBarChart
        data={data}
        singleAppId={appId}
        periodTicks={durationToTicks(period)}
        rangeMinTicks={dateTimeToTicks(start)}
        rangeMaxTicks={dateTimeToTicks(end)}
        dateTimeFormatter={formatter}
        gradientBars
        className="aspect-auto h-full flex-1"
        maxYIsPeriod
      />
    </div>
  );
}

export default function App({ params }: Route.ComponentProps) {
  const id = +params.id;
  const app = useAppState((state) => state.apps[id as Ref<App>])!;
  const removeAppTag = useAppState((state) => state.removeAppTag);
  const updateApp = useAppState((state) => state.updateApp);

  const allTags = useAppState((state) => state.tags);
  const { handleStaleTags } = useRefresh();
  const tags = useMemo(
    () =>
      _(app.tags)
        .map((tagId) => allTags[tagId])
        .thru(handleStaleTags)
        .value(),
    [handleStaleTags, allTags, app.tags],
  );

  const today = useToday();
  const [dayStart, dayEnd, dayPeriod, dayFormatter] = useMemo(
    () => [
      today.startOf("day"),
      today.endOf("day"),
      Duration.fromObject({ hour: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("HHmm"),
    ],
    [today],
  );
  const [weekStart, weekEnd, weekPeriod, weekFormatter] = useMemo(
    () => [
      today.startOf("week"),
      today.endOf("week"),
      Duration.fromObject({ day: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("EEE"),
    ],
    [today],
  );
  const [monthStart, monthEnd, monthPeriod, monthFormatter] = useMemo(
    () => [
      today.startOf("month"),
      today.endOf("month"),
      Duration.fromObject({ day: 1 }),
      (dateTime: DateTime) => dateTime.toFormat("dd"),
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
                <EditableText
                  text={app.name}
                  className="text-2xl font-semibold grow-0"
                  buttonClassName="ml-1"
                  onSubmit={async (v) => await updateApp({ ...app, name: v })}
                />
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

            {/* Tags */}
            {tags.length > 0 && (
              <HorizontalOverflowList
                className="gap-2 max-w-full h-8"
                items={tags}
                renderItem={(tag) => (
                  <MiniTagItem
                    key={tag.id}
                    tag={tag}
                    className="h-8"
                    remove={async () => await removeAppTag(app.id, tag.id)}
                  />
                )}
                renderOverflowItem={(tag) => (
                  <MiniTagItem key={tag.id} tag={tag} className="h-8" />
                )}
                renderOverflowSign={(items) => (
                  <Badge
                    variant="outline"
                    style={{
                      backgroundColor: "rgba(255, 255, 255, 0.2)",
                    }}
                    className="whitespace-nowrap ml-2 text-muted-foreground rounded-md h-8"
                  >{`+${items.length}`}</Badge>
                )}
              />
            )}

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
            title="Today"
            usage={app.usages.usage_today}
            totalUsage={0}
            start={dayStart}
            end={dayEnd}
            period={dayPeriod}
            appId={app.id}
            formatter={dayFormatter}
          />
          <CardUsage
            title="Week"
            usage={app.usages.usage_week}
            totalUsage={0}
            start={weekStart}
            end={weekEnd}
            period={weekPeriod}
            appId={app.id}
            formatter={weekFormatter}
          />
          <CardUsage
            title="Month"
            usage={app.usages.usage_month}
            totalUsage={0}
            start={monthStart}
            end={monthEnd}
            period={monthPeriod}
            appId={app.id}
            formatter={monthFormatter}
          />
        </div>
        <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min" />
      </div>
    </>
  );
}

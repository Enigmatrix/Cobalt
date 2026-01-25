import AppIcon from "@/components/app/app-icon";
import { NoApps, NoAppsFound } from "@/components/empty-states";
import { SearchBar } from "@/components/search-bar";
import { ScoreCircle } from "@/components/tag/score";
import { DurationText } from "@/components/time/duration-text";
import { Badge } from "@/components/ui/badge";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { Text } from "@/components/ui/text";
import { UsageChart } from "@/components/viz/usage-chart";
import { useHistoryRef, useHistoryState } from "@/hooks/use-history-state";
import { useApps, useTag } from "@/hooks/use-refresh";
import { useAppDurationsPerPeriod } from "@/hooks/use-repo";
import { useAppsSearch } from "@/hooks/use-search";
import { SortDirection } from "@/hooks/use-sort";
import { usePeriodInterval } from "@/hooks/use-time";
import type { App, Tag, WithGroupedDuration } from "@/lib/entities";
import type { EntityMap } from "@/lib/state";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import _ from "lodash";
import { ArrowDownUp, SortAsc, SortDesc } from "lucide-react";
import { DateTime } from "luxon";
import {
  memo,
  useCallback,
  useMemo,
  type CSSProperties,
  type ReactNode,
} from "react";
import { NavLink } from "react-router";
import AutoSizer from "react-virtualized-auto-sizer";
import { FixedSizeList as List, type ListOnScrollProps } from "react-window";

export function MiniTagItem({
  tag,
  className,
}: {
  tag: Tag;
  className?: ClassValue;
}) {
  return (
    <Badge
      variant="outline"
      style={{
        borderColor: tag.color,
        color: tag.color,
        backgroundColor: "rgba(255, 255, 255, 0.2)",
      }}
      className={cn("whitespace-nowrap min-w-0", className)}
    >
      <Text className="max-w-32">{tag.name}</Text>
      <ScoreCircle score={tag.score} className="ml-2 -mr-1" />
    </Badge>
  );
}

type SortProperty =
  | "name"
  | "company"
  | "usages.today"
  | "usages.week"
  | "usages.month";

export default function Apps() {
  const apps = useApps();

  const [sortDirection, setSortDirection] = useHistoryState<SortDirection>(
    SortDirection.Descending,
    "sortDirection",
  );
  const [sortProperty, setSortProperty] = useHistoryState<SortProperty>(
    "usages.today",
    "sortProperty",
  );

  const [search, setSearch, appsFiltered] = useAppsSearch(apps, "query");
  const appsSorted = useMemo(() => {
    return _(appsFiltered).orderBy([sortProperty], [sortDirection]).value();
  }, [appsFiltered, sortDirection, sortProperty]);

  const interval = usePeriodInterval("day");
  const { ret: appDurationsPerPeriod } = useAppDurationsPerPeriod({
    ...interval,
    period: "hour",
  });

  const [scrollOffset, setScrollOffset] = useHistoryRef<number>(
    0,
    "scrollOffset",
  );

  const setScrollOffsetFromEvent = useCallback(
    (e: ListOnScrollProps) => {
      setScrollOffset(e.scrollOffset);
    },
    [setScrollOffset],
  );

  const ListItem = memo(
    ({ index, style }: { index: number; style: CSSProperties }) => (
      <VirtualListItem style={style}>
        <AppListItem
          app={appsSorted[index]}
          start={interval.start}
          end={interval.end}
          appDurationsPerPeriod={appDurationsPerPeriod}
        />
      </VirtualListItem>
    ),
  );

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbPage>Apps</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>

        <div className="flex-1" />

        <SearchBar
          className="max-w-80"
          placeholder="Search..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="icon" className="shrink-0">
              <ArrowDownUp size={16} />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuRadioGroup
              value={sortDirection}
              onValueChange={(v) => setSortDirection(v as SortDirection)}
            >
              <DropdownMenuRadioItem value={SortDirection.Ascending}>
                <div className="inline-flex items-center gap-2">
                  <SortAsc size={16} />
                  <span>Ascending</span>
                </div>
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value={SortDirection.Descending}>
                <div className="inline-flex items-center gap-2">
                  <SortDesc size={16} />
                  <span>Descending</span>
                </div>
              </DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
            <DropdownMenuSeparator />
            <DropdownMenuRadioGroup
              value={sortProperty}
              onValueChange={(v) => setSortProperty(v as SortProperty)}
            >
              <DropdownMenuRadioItem value="name">Name</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="company">
                Company
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="usages.today">
                Usage Today
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="usages.week">
                Usage Week
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="usages.month">
                Usage Month
              </DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </header>

      <div className="flex flex-1 flex-col max-w-full overflow-hidden">
        <AutoSizer>
          {({ height, width }) => (
            <List
              initialScrollOffset={scrollOffset}
              onScroll={setScrollOffsetFromEvent}
              itemData={appsSorted} // this is to trigger a rerender on sort/filter
              height={height}
              width={width}
              itemSize={96} // 24rem, manually calculated
              itemCount={appsSorted.length}
            >
              {ListItem}
            </List>
          )}
        </AutoSizer>
        {apps.length === 0 && <NoApps className="m-auto" />}
        {apps.length !== 0 && appsSorted.length === 0 && (
          <NoAppsFound className="m-auto" />
        )}
      </div>
    </>
  );
}

// Virtual list item for react-window. Actual height in style
// is ignored, instead we use h-24 and h-28 (if last, to show bottom gap).
// There is a h-4 gap at the top of each item.
function VirtualListItem({
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  style: { height, ...style },
  children,
}: {
  style: CSSProperties;
  children?: ReactNode;
}) {
  return (
    // Due to how virtualization works, the last:h-28 triggers for the last item
    // in the *window*, it just happens that due to overscanning that last item
    // is hidden unless it's the actual last item in the list.
    <div className="flex flex-col px-4 h-24 last:h-28" style={style}>
      {/* gap */}
      <div className="h-4" />
      {children}
    </div>
  );
}

function AppListItem({
  app,
  appDurationsPerPeriod,
  start,
  end,
}: {
  app: App;
  appDurationsPerPeriod: EntityMap<App, WithGroupedDuration<App>[]>;
  start: DateTime;
  end: DateTime;
}) {
  const tag = useTag(app.tagId);

  return (
    <NavLink
      to={`/apps/${app.id}`}
      className={cn(
        "h-20 shadow-xs rounded-md flex items-center gap-2 p-4 @container",
        "ring-offset-background transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        "disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none",
        "bg-card text-card-foreground hover:bg-muted/75 border-border border",
      )}
    >
      <AppIcon app={app} className="mx-2 h-10 w-10 shrink-0" />

      <div className="flex flex-col min-w-0">
        <div className="inline-flex items-center gap-2">
          <Text className="text-lg font-semibold max-w-72">{app.name}</Text>
          {tag && <MiniTagItem tag={tag} />}
        </div>
        <span className="inline-flex gap-1 items-center text-xs text-card-foreground/50">
          {app.identity.tag !== "website" && (
            <Text className="max-w-48">{app.company}</Text>
          )}
          {app.description && (
            <>
              {app.identity.tag !== "website" && <p>|</p>}
              <Text className="max-w-[40rem]">{app.description}</Text>
            </>
          )}
        </span>
      </div>

      <div className="flex-1" />

      {app.usages.today > 0 ? (
        <>
          <UsageChart
            hideXAxis
            hideYAxis
            gradientBars
            maxYIsPeriod
            appDurationsPerPeriod={appDurationsPerPeriod}
            period="hour"
            onlyShowOneApp={app.id}
            start={start}
            end={end}
            className="w-48 flex-none aspect-none h-20"
          />

          <div className="flex py-2 rounded-md lg:min-w-20">
            <div className="flex flex-col items-end ml-auto my-auto">
              <div className="text-xs text-card-foreground/50">Today</div>
              <DurationText
                className="text-lg min-w-8 text-center whitespace-nowrap"
                ticks={app.usages.today}
              />
            </div>
          </div>
        </>
      ) : null}
    </NavLink>
  );
}

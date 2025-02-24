import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import type { EntityMap } from "@/lib/state";
import type { App, Tag, WithGroupedDuration } from "@/lib/entities";
import {
  memo,
  useMemo,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";
import { Badge } from "@/components/ui/badge";
import { FixedSizeList as List } from "react-window";
import AutoSizer from "react-virtualized-auto-sizer";
import { cn } from "@/lib/utils";
import { NavLink } from "react-router";
import AppIcon from "@/components/app/app-icon";
import { SearchBar } from "@/components/search-bar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { ArrowDownUp, SortAsc, SortDesc } from "lucide-react";
import _ from "lodash";
import { Text } from "@/components/ui/text";
import { dateTimeToTicks, durationToTicks, hour } from "@/lib/time";
import { DateTime, Duration } from "luxon";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import { useToday } from "@/hooks/use-today";
import { useApps, useTag } from "@/hooks/use-refresh";
import { useAppsSearch } from "@/hooks/use-search";
import { DurationText } from "@/components/time/duration-text";
import type { ClassValue } from "clsx";
import { useAppDurationsPerPeriod } from "@/hooks/use-repo";
import { SortDirection } from "@/hooks/use-sort";

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
  const today = useToday();
  const apps = useApps();

  const [sortDirection, setSortDirection] = useState<SortDirection>(
    SortDirection.Descending,
  );
  const [sortProperty, setSortProperty] =
    useState<SortProperty>("usages.today");

  const [search, setSearch, appsFiltered] = useAppsSearch(apps);
  const appsSorted = useMemo(() => {
    return _(appsFiltered).orderBy([sortProperty], [sortDirection]).value();
  }, [appsFiltered, sortDirection, sortProperty]);

  const [start, end] = useMemo(() => [today, today.endOf("day")], [today]);
  const period = hour;
  const {
    usages,
    start: loadStart,
    end: loadEnd,
  } = useAppDurationsPerPeriod({ start, end, period });

  const ListItem = memo(
    ({ index, style }: { index: number; style: CSSProperties }) => (
      <VirtualListItem style={style}>
        <AppListItem
          app={appsSorted[index]}
          start={loadStart ?? start}
          end={loadEnd ?? end}
          period={period}
          usages={usages}
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

        <div className="xl:flex-1 flex-none" />

        <SearchBar
          className="max-w-80 m-auto xl:m-0"
          placeholder="Search..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon">
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
  usages,
  start,
  end,
  period,
}: {
  app: App;
  usages: EntityMap<App, WithGroupedDuration<App>[]>;
  start: DateTime;
  end: DateTime;
  period: Duration;
}) {
  const tag = useTag(app.tag_id);

  return (
    <NavLink
      to={`/apps/${app.id}`}
      className={cn(
        "h-20 shadow-sm rounded-md flex items-center gap-2 p-4 @container",
        "ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        "disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none cursor-pointer",
        "bg-card text-card-foreground hover:bg-muted/75 border-border border",
      )}
    >
      <AppIcon buffer={app.icon} className="mx-2 h-10 w-10 flex-shrink-0" />

      <div className="flex flex-col min-w-0">
        <div className="inline-flex items-center gap-2">
          <Text className="text-lg font-semibold max-w-72">{app.name}</Text>
          {tag && <MiniTagItem tag={tag} />}
        </div>
        <span className="inline-flex gap-1 items-center text-xs text-card-foreground/50">
          <Text className="max-w-48">{app.company}</Text>
          {app.description && (
            <>
              <p>|</p>
              <Text className="max-w-[40rem]">{app.description}</Text>
            </>
          )}
        </span>
      </div>

      <div className="flex-1" />

      {app.usages.today > 0 ? (
        <>
          <AppUsageBarChart
            hideXAxis
            gradientBars
            maxYIsPeriod
            data={usages}
            singleAppId={app.id}
            rangeMinTicks={dateTimeToTicks(start)}
            rangeMaxTicks={dateTimeToTicks(end)}
            periodTicks={durationToTicks(period)}
            className="w-48 aspect-auto h-20 max-lg:hidden"
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

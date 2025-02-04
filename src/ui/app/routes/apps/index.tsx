import { SidebarTrigger } from "@/components/ui/sidebar";
import useResizeObserver from "@react-hook/resize-observer";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import { useAppState, type EntityMap } from "@/lib/state";
import type { App, Tag, WithGroupedDuration } from "@/lib/entities";
import {
  memo,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";
import { Badge } from "@/components/ui/badge";
import { FixedSizeList as List } from "react-window";
import AutoSizer from "react-virtualized-auto-sizer";
import { cn } from "@/lib/utils";
import { NavLink } from "react-router";
import AppIcon from "@/components/app-icon";
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
import { ArrowDownUp, Filter, SortAsc, SortDesc } from "lucide-react";
import _ from "lodash";
import fuzzysort from "fuzzysort";
import { Text } from "@/components/ui/text";
import type { ClassValue } from "clsx";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { dateTimeToTicks, durationToTicks, toHumanDuration } from "@/lib/time";
import { getAppDurationsPerPeriod } from "@/lib/repo";
import { DateTime, Duration } from "luxon";
import { AppUsageBarChart } from "@/components/app-usage-chart";

function TagItem({ tag }: { tag: Tag }) {
  return (
    <Badge
      variant="outline"
      style={{
        borderColor: tag.color,
        color: tag.color,
        backgroundColor: "rgba(255, 255, 255, 0.2)",
      }}
      className="whitespace-nowrap"
    >
      <Text className="max-w-32">{tag.name}</Text>
    </Badge>
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

function useWidth(
  target: React.RefObject<HTMLElement | null>,
  initWidth?: number
) {
  const [width, setWidth] = useState(initWidth || 0);

  useLayoutEffect(() => {
    if (!target?.current) return;
    setWidth(target.current.getBoundingClientRect().width);
  }, [target]);

  // Where the magic happens
  useResizeObserver(target, (entry) => setWidth(entry.contentRect.width));
  return width;
}

// Assumes rendered items are same height. Height of this container
// must be manually set to height of the rendered item.
function HorizontalOverflowList<T>({
  className,
  items,
  renderItem,
  renderOverflowItem,
  renderOverflowSign,
}: {
  className: ClassValue;
  items: T[];
  renderItem: (item: T) => ReactNode;
  renderOverflowItem: (item: T) => ReactNode;
  renderOverflowSign: (overflowItems: T[]) => ReactNode;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const width = useWidth(ref);
  useLayoutEffect(() => {
    if (!ref.current) return;
    const items = ref.current.children;
    const firstItem = items.item(0) as HTMLElement;
    const offsetLeft = firstItem?.offsetLeft;
    const offsetTop = firstItem?.offsetTop;
    const overflowIndex = _.findIndex(
      items,
      (item) =>
        (item as HTMLElement).offsetLeft === offsetLeft &&
        (item as HTMLElement).offsetTop !== offsetTop
    );
    setOverflowIndex(overflowIndex);
  }, [width, ref]);
  const [overflowIndex, setOverflowIndex] = useState(0);
  const overflowItems = useMemo(
    () => items.slice(overflowIndex),
    [items, overflowIndex]
  );

  return (
    <div
      ref={ref}
      className={cn("flex flex-wrap items-center overflow-hidden", className)}
    >
      {items.map((item, index) => (
        <div className="flex items-center" key={index}>
          {renderItem(item)}

          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger
                asChild
                className={cn({
                  hidden: index !== overflowIndex - 1,
                })}
              >
                {renderOverflowSign(overflowItems)}
              </TooltipTrigger>
              <TooltipContent>
                <div className="flex flex-col gap-2 py-2">
                  {overflowItems.map(renderOverflowItem)}
                </div>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      ))}
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
  const allTags = useAppState((state) => state.tags);
  const tags = useMemo(
    () =>
      app.tags
        .map((tagId) => allTags[tagId])
        .filter((tag) => tag !== undefined), // filter stale tags
    [allTags, app.tags]
  );
  const singleUsage = useMemo(
    () => ({ [app.id]: usages[app.id] }),
    [usages, app.id]
  );
  return (
    <NavLink
      to={`/apps/${app.id}`}
      className={cn(
        "h-20 shadow-sm rounded-md flex items-center gap-2 p-4 @container",
        "ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        "disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none cursor-pointer",
        "bg-primary-foreground text-primary hover:bg-primary/10 border-border border"
      )}
    >
      <AppIcon buffer={app.icon} className="mx-2 h-10 w-10 flex-shrink-0" />

      <div className="flex flex-col min-w-0">
        <div className="inline-flex items-center gap-2">
          <Text className="text-lg font-semibold max-w-72">{app.name}</Text>
          <HorizontalOverflowList
            className="gap-1 max-w-96 h-6"
            items={tags}
            renderItem={(tag) => <TagItem key={tag.id} tag={tag} />}
            renderOverflowItem={(tag) => <TagItem key={tag.id} tag={tag} />}
            renderOverflowSign={(items) => (
              <Badge
                variant="outline"
                style={{
                  backgroundColor: "rgba(255, 255, 255, 0.2)",
                }}
                className="whitespace-nowrap ml-1 text-primary/60 rounded-md"
              >{`+${items.length}`}</Badge>
            )}
          />
        </div>
        <span className="inline-flex gap-1 items-center text-xs text-primary/50">
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

      {app.usages.usage_today > 0 ? (
        <>
          <AppUsageBarChart
            hideXAxis
            gradientBars
            maxYIsPeriod
            data={singleUsage}
            singleAppId={app.id}
            rangeMinTicks={dateTimeToTicks(start)}
            rangeMaxTicks={dateTimeToTicks(end)}
            periodTicks={durationToTicks(period)}
            className="min-w-48 aspect-auto h-20 max-lg:hidden"
          />

          <div className="flex py-2 rounded-md lg:min-w-20">
            <div className="flex flex-col items-end ml-auto my-auto">
              <div className="text-xs text-primary/50">Today</div>
              <div className="text-base min-w-8 text-center">
                {toHumanDuration(app.usages.usage_today)}
              </div>
            </div>
          </div>
        </>
      ) : null}
    </NavLink>
  );
}

enum SortDirection {
  Ascending = "asc",
  Descending = "desc",
}

type SortProperty =
  | "name"
  | "company"
  | "usages.usage_today"
  | "usages.usage_week"
  | "usages.usage_month";

export default function Apps() {
  const apps = useAppState((state) => state.apps);
  const [sortDirection, setSortDirection] = useState<SortDirection>(
    SortDirection.Descending
  );
  const [sortProperty, setSortProperty] =
    useState<SortProperty>("usages.usage_today");
  const [search, setSearch] = useState<string>("");

  const appValues = useMemo(() => Object.values(apps), [apps]);

  const appsFiltered = useMemo(() => {
    if (search) {
      const results = fuzzysort.go(search, appValues, {
        keys: ["name", "company", "description"],
      });
      // ignore fuzzy-search sorting.
      return _.map(results, "obj");
    } else {
      return appValues;
    }
  }, [appValues, search]);

  const appsSorted = useMemo(() => {
    return _(appsFiltered)
      .orderBy([sortProperty], [sortDirection])
      .filter((app) => app !== undefined) // filter stale apps
      .value();
  }, [appsFiltered, sortDirection, sortProperty]);

  const today = useMemo(() => DateTime.now().startOf("day"), []);
  const period = useMemo(() => Duration.fromObject({ hour: 1 }), []);
  const [start, end] = useMemo(
    () => [today, today.endOf("day")],
    [today, period]
  );
  const [usages, setUsages] = useState<
    EntityMap<App, WithGroupedDuration<App>[]>
  >({});
  useEffect(() => {
    getAppDurationsPerPeriod({
      start,
      end,
      period,
    }).then((usages) => setUsages(usages));
  }, [start, end, period]);

  const ListItem = memo(
    ({ index, style }: { index: number; style: CSSProperties }) => (
      <VirtualListItem style={style}>
        <AppListItem
          app={appsSorted[index]}
          start={start}
          end={end}
          period={period}
          usages={usages}
        />
      </VirtualListItem>
    )
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
              <DropdownMenuRadioItem value="usages.usage_today">
                Usage Today
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="usages.usage_week">
                Usage Week
              </DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="usages.usage_month">
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

import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import type { Tag, App, WithGroupedDuration } from "@/lib/entities";
import { Text } from "@/components/ui/text";
import { cn } from "@/lib/utils";
import type { ClassValue } from "clsx";
import { DateTime, Duration } from "luxon";
import AppIcon from "@/components/app/app-icon";
import {
  memo,
  useMemo,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";
import { DurationText } from "@/components/time/duration-text";
import { useApps, useTags } from "@/hooks/use-refresh";
import type { EntityMap } from "@/lib/state";
import { NavLink } from "react-router";
import { ArrowDownUp, Plus, SortAsc, SortDesc, TagIcon } from "lucide-react";
import { useAppDurationsPerPeriod } from "@/hooks/use-repo";
import { useSearch } from "@/hooks/use-search";
import { useToday } from "@/hooks/use-today";
import AutoSizer from "react-virtualized-auto-sizer";
import { FixedSizeList as List } from "react-window";
import { SearchBar } from "@/components/search-bar";
import { HorizontalOverflowList } from "@/components/overflow-list";
import { Badge } from "@/components/ui/badge";
import { SortDirection } from "@/hooks/use-sort";
import _ from "lodash";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { dateTimeToTicks, durationToTicks } from "@/lib/time";
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import { CreateTagDialog } from "@/components/tag/create-tag-dialog";
import { createTag } from "@/lib/repo";

const period = Duration.fromObject({ hour: 1 });

export function MiniAppItem({
  app,
  className,
}: {
  app: App;
  className?: ClassValue;
}) {
  return (
    <div
      className={cn(
        "whitespace-nowrap min-w-0 flex gap-2 py-1 px-2 rounded-md border border-border bg-muted/50",
        className,
      )}
    >
      <AppIcon buffer={app?.icon} className="w-4 h-4" />
      <Text className="max-w-52 text-sm">{app?.name ?? ""}</Text>
    </div>
  );
}

// Virtual list item for react-window. Actual height in style
// is ignored, instead we use h-28 and h-32 (if last, to show bottom gap).
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
    <div className="flex flex-col px-4 h-28 last:h-32" style={style}>
      {/* gap */}
      <div className="h-4" />
      {children}
    </div>
  );
}

function TagListItem({
  tag,
  usages,
  start,
  end,
  period,
}: {
  tag: Tag;
  usages: EntityMap<App, WithGroupedDuration<App>[]>;
  start: DateTime;
  end: DateTime;
  period: Duration;
}) {
  const apps = useApps(tag.apps);
  const usagesFiltered = useMemo(() => {
    return _(apps)
      .map((app) => [app.id, usages[app.id]])
      .fromPairs()
      .value();
  }, [usages, apps]);
  return (
    <NavLink
      to={`/tags/${tag.id}`}
      className={cn(
        "h-24 shadow-sm rounded-md flex items-center gap-2 p-4 @container",
        "ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        "disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none cursor-pointer",
        "bg-card text-card-foreground hover:bg-muted/75 border-border border",
      )}
    >
      <TagIcon
        className="mx-2 h-10 w-10 flex-shrink-0"
        style={{ color: tag.color }}
      />

      <div className="flex flex-col min-w-0 gap-1">
        <Text className="text-lg font-semibold max-w-72">{tag.name}</Text>
        {apps.length !== 0 && (
          <HorizontalOverflowList
            className="gap-1 h-8"
            items={apps}
            renderItem={(app) => <MiniAppItem key={app.id} app={app} />}
            renderOverflowItem={(app) => <MiniAppItem key={app.id} app={app} />}
            renderOverflowSign={(items) => (
              <Badge
                variant="outline"
                style={{
                  backgroundColor: "rgba(255, 255, 255, 0.2)",
                }}
                className="whitespace-nowrap ml-1 text-card-foreground/60 rounded-md h-8"
              >{`+${items.length}`}</Badge>
            )}
          />
        )}
      </div>

      <div className="flex-1" />

      {tag.usages.usage_today > 0 ? (
        <>
          <AppUsageBarChart
            hideXAxis
            gradientBars
            maxYIsPeriod
            data={usagesFiltered}
            rangeMinTicks={dateTimeToTicks(start)}
            rangeMaxTicks={dateTimeToTicks(end)}
            periodTicks={durationToTicks(period)}
            className="min-w-48 aspect-auto h-20 max-lg:hidden"
          />

          <div className="flex py-2 rounded-md lg:min-w-20">
            <div className="flex flex-col items-end ml-auto my-auto">
              <div className="text-xs text-card-foreground/50">Today</div>
              <DurationText
                className="text-lg min-w-8 text-center whitespace-nowrap"
                ticks={tag.usages.usage_today}
              />
            </div>
          </div>
        </>
      ) : null}
    </NavLink>
  );
}

type SortProperty =
  | "name"
  | "usages.usage_today"
  | "usages.usage_week"
  | "usages.usage_month";

export default function Tags() {
  const today = useToday();
  const tags = useTags();

  const [sortDirection, setSortDirection] = useState<SortDirection>(
    SortDirection.Descending,
  );
  const [sortProperty, setSortProperty] =
    useState<SortProperty>("usages.usage_today");
  const [search, setSearch, tagsFiltered] = useSearch(tags, ["name"]);
  const tagsSorted = useMemo(() => {
    return _(tagsFiltered).orderBy([sortProperty], [sortDirection]).value();
  }, [tagsFiltered, sortDirection, sortProperty]);

  const [start, end] = useMemo(() => [today, today.endOf("day")], [today]);
  const {
    usages,
    start: loadStart,
    end: loadEnd,
  } = useAppDurationsPerPeriod({ start, end, period });
  const ListItem = memo(
    ({ index, style }: { index: number; style: CSSProperties }) => (
      <VirtualListItem style={style}>
        <TagListItem
          tag={tagsSorted[index]}
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
              <BreadcrumbPage>Tags</BreadcrumbPage>
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

        <CreateTagDialog
          trigger={
            <Button variant="outline">
              <Plus />
              Create Tag
            </Button>
          }
          onSubmit={async (tag) => {
            await createTag(tag);
          }}
        ></CreateTagDialog>
      </header>

      <div className="flex flex-1 flex-col max-w-full overflow-hidden">
        <AutoSizer>
          {({ height, width }) => (
            <List
              itemData={tagsSorted} // this is to trigger a rerender on sort/filter
              height={height}
              width={width}
              itemSize={112} // 28rem, manually calculated
              itemCount={tagsSorted.length}
            >
              {ListItem}
            </List>
          )}
        </AutoSizer>
      </div>
    </>
  );
}

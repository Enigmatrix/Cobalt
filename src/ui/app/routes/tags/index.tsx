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
import { DateTime } from "luxon";
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
import { useAppState, type EntityMap } from "@/lib/state";
import { NavLink } from "react-router";
import { ArrowDownUp, Plus, SortAsc, SortDesc, TagIcon } from "lucide-react";
import { useAppDurationsPerPeriod } from "@/hooks/use-repo";
import { useTagsSearch } from "@/hooks/use-search";
import { usePeriodInterval } from "@/hooks/use-today";
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
import { AppUsageBarChart } from "@/components/viz/app-usage-chart";
import { CreateTagDialog } from "@/components/tag/create-tag-dialog";
import { NoTags, NoTagsFound } from "@/components/empty-states";

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
        "whitespace-nowrap min-w-0 flex gap-2 py-1 px-2 rounded-md border border-border h-6 items-center bg-secondary",
        className,
      )}
      style={{
        backgroundColor: "rgba(255, 255, 255, 0.1)",
      }}
    >
      <AppIcon buffer={app?.icon} className="w-4 h-4" />
      <Text className="max-w-52 text-xs">{app?.name ?? ""}</Text>
    </div>
  );
}

type SortProperty = "name" | "usages.today" | "usages.week" | "usages.month";

export default function Tags() {
  const tags = useTags();
  const createTag = useAppState((state) => state.createTag);

  const [sortDirection, setSortDirection] = useState<SortDirection>(
    SortDirection.Descending,
  );
  const [sortProperty, setSortProperty] =
    useState<SortProperty>("usages.today");
  const [search, setSearch, tagsFiltered] = useTagsSearch(tags);
  const tagsSorted = useMemo(() => {
    return _(tagsFiltered).orderBy([sortProperty], [sortDirection]).value();
  }, [tagsFiltered, sortDirection, sortProperty]);

  const range = usePeriodInterval("day");
  const {
    usages,
    start: loadStart,
    end: loadEnd,
  } = useAppDurationsPerPeriod({ ...range, period: "hour" });
  const ListItem = memo(
    ({ index, style }: { index: number; style: CSSProperties }) => (
      <VirtualListItem style={style}>
        <TagListItem
          tag={tagsSorted[index]}
          start={loadStart ?? range.start}
          end={loadEnd ?? range.end}
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
              itemSize={96} // 24rem, manually calculated
              itemCount={tagsSorted.length}
            >
              {ListItem}
            </List>
          )}
        </AutoSizer>
        {tags.length === 0 && <NoTags className="m-auto" />}
        {tags.length !== 0 && tagsSorted.length === 0 && (
          <NoTagsFound className="m-auto" />
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

function TagListItem({
  tag,
  usages,
  start,
  end,
}: {
  tag: Tag;
  usages: EntityMap<App, WithGroupedDuration<App>[]>;
  start: DateTime;
  end: DateTime;
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
        "h-20 shadow-xs rounded-md flex items-center gap-2 p-4 @container",
        "ring-offset-background transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        "disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none",
        "bg-card text-card-foreground hover:bg-muted/75 border-border border",
      )}
    >
      <TagIcon
        className="mx-2 h-10 w-10 shrink-0"
        style={{ color: tag.color }}
      />

      <div className="flex flex-col min-w-0 gap-1">
        <Text className="text-lg font-semibold max-w-72">{tag.name}</Text>
        {apps.length !== 0 && (
          <HorizontalOverflowList
            className="gap-1 h-6 -mb-2"
            items={apps}
            renderItem={(app) => <MiniAppItem key={app.id} app={app} />}
            renderOverflowItem={(app) => <MiniAppItem key={app.id} app={app} />}
            renderOverflowSign={(items) => (
              <Badge
                variant="outline"
                style={{
                  backgroundColor: "rgba(255, 255, 255, 0.2)",
                }}
                className="whitespace-nowrap ml-1 text-card-foreground/60 rounded-md h-6"
              >{`+${items.length}`}</Badge>
            )}
          />
        )}
      </div>

      <div className="flex-1" />

      {tag.usages.today > 0 ? (
        <>
          <AppUsageBarChart
            hideXAxis
            hideYAxis
            gradientBars
            maxYIsPeriod
            data={usagesFiltered}
            start={start}
            end={end}
            period="hour"
            className="w-48 flex-none aspect-none h-20 max-lg:hidden"
          />

          <div className="flex py-2 rounded-md lg:min-w-20">
            <div className="flex flex-col items-end ml-auto my-auto">
              <div className="text-xs text-card-foreground/50">Today</div>
              <DurationText
                className="text-lg min-w-8 text-center whitespace-nowrap"
                ticks={tag.usages.today}
              />
            </div>
          </div>
        </>
      ) : null}
    </NavLink>
  );
}

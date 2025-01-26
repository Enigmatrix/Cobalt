import { SidebarTrigger } from "@/components/ui/sidebar";
import useResizeObserver from "@react-hook/resize-observer";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import { useAppState } from "@/lib/state";
import type { App, Ref, Tag } from "@/lib/entities";
import {
  memo,
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
import { Input } from "@/components/ui/input";
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

function TagItem({ tagId }: { tagId: Ref<Tag> }) {
  const tag = useAppState((state) => state.tags[tagId]); // TODO check if this even works

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

          <div
            className={cn({
              hidden: index !== overflowIndex - 1,
            })}
          >
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger>
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
        </div>
      ))}
    </div>
  );
}

function AppListItem({ app }: { app: App }) {
  return (
    <NavLink
      to={`/apps/${app.id}`}
      className={cn(
        "h-20 shadow-sm rounded-md flex items-center gap-2 p-4",
        "ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        "disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none cursor-pointer",
        "bg-secondary text-secondary-foreground hover:bg-secondary/80"
      )}
    >
      <AppIcon buffer={app.icon} className="mx-2 h-10 w-10 flex-shrink-0" />

      <div className="flex flex-col min-w-0">
        <div className="inline-flex items-center gap-2">
          <Text className="text-lg font-semibold max-w-72">{app.name}</Text>
          <HorizontalOverflowList
            className="gap-1 max-w-96 h-6"
            items={app.tags}
            renderItem={(tagId) => <TagItem key={tagId} tagId={tagId} />}
            renderOverflowItem={(tagId) => (
              <TagItem key={tagId} tagId={tagId} />
            )}
            renderOverflowSign={(items) => (
              <Badge className="whitespace-nowrap ml-1 bg-white/20 hover:bg-white/30 text-white/60 rounded-md">{`+${items.length}`}</Badge>
            )}
          />
        </div>
        <span className="inline-flex gap-1 items-center text-white/50 text-xs">
          <Text className="max-w-48">{app.company}</Text>
          {app.description && (
            <>
              <p>|</p>
              <Text className="max-w-[40rem]">{app.description}</Text>
            </>
          )}
        </span>
      </div>
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
    return _.orderBy(appsFiltered, [sortProperty], [sortDirection]);
  }, [appsFiltered, sortDirection, sortProperty]);

  const ListItem = memo(
    ({ index, style }: { index: number; style: CSSProperties }) => (
      <VirtualListItem style={style}>
        <AppListItem app={appsSorted[index]} />
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

        <div className="flex-1" />

        <Input
          className=" max-w-80"
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

import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import { useAppState } from "@/lib/state";
import type { App, Ref, Tag } from "@/lib/entities";
import { useMemo, useState, type CSSProperties, type ReactNode } from "react";
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
    >
      {tag.name}
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

function AppListItem({ app }: { app: App }) {
  return (
    <NavLink
      to={`/apps/${app.id}`}
      className={cn(
        "h-20 shadow-sm rounded-md flex overflow-auto",
        "ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        "disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none cursor-pointer",
        "bg-secondary text-secondary-foreground hover:bg-secondary/80"
      )}
    >
      <div className="flex-1 flex items-center gap-2 p-4">
        <AppIcon buffer={app.icon} className="mx-2 h-10 w-10" />

        <div className="flex flex-col">
          <div className="inline-flex items-center gap-2">
            <div className="text-lg font-semibold max-w-72 truncate">
              {app.name}
            </div>
            <>
              {app.tags.map((tagId) => (
                <TagItem key={tagId} tagId={tagId} />
              ))}
            </>
          </div>
          <span className="inline-flex gap-1 items-center text-white/50 text-xs">
            <p className="max-w-48 truncate">{app.company}</p>
            {app.description && (
              <>
                <p>|</p>
                <p className=" max-w-[40rem] truncate">{app.description}</p>
              </>
            )}
          </span>
        </div>
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

  const appsSorted = useMemo(() => {
    const appValues = Object.values(apps);
    return _.orderBy(appValues, [sortProperty], [sortDirection]);
  }, [apps, sortDirection, sortProperty]);

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

        <Input className=" max-w-80" placeholder="Search..." />

        <DropdownMenu>
          <DropdownMenuTrigger>
            <Button variant="ghost">
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
              {({ index, style }) => (
                <VirtualListItem style={style}>
                  <AppListItem app={appsSorted[index]} />
                </VirtualListItem>
              )}
            </List>
          )}
        </AutoSizer>
      </div>
    </>
  );
}

import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import type { Alert, Reminder } from "@/lib/entities";
import { Text } from "@/components/ui/text";
import { cn } from "@/lib/utils";
import AppIcon from "@/components/app/app-icon";
import { memo, useMemo, type CSSProperties, type ReactNode } from "react";
import { DurationText } from "@/components/time/duration-text";
import { useApps, useAlerts, useApp, useTag } from "@/hooks/use-refresh";
import { NavLink } from "react-router";
import { Plus, TagIcon } from "lucide-react";
import AutoSizer from "react-virtualized-auto-sizer";
import { FixedSizeList as List } from "react-window";
import { HorizontalOverflowList } from "@/components/overflow-list";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { MiniAppItem } from "@/routes/tags";
import { MiniTagItem } from "@/routes/apps";
import { useAlertsSearch } from "@/hooks/use-search";
import { SearchBar } from "@/components/search-bar";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { ClassValue } from "clsx";
import { NoAlerts, NoAlertsFound } from "@/components/empty-states";

export default function Alerts() {
  const alerts = useAlerts();
  const [search, setSearch, alertsFiltered] = useAlertsSearch(alerts);
  const alertsSorted = alertsFiltered; // TODO: sort

  const ListItem = memo(
    ({ index, style }: { index: number; style: CSSProperties }) => (
      <VirtualListItem style={style}>
        <AlertListItem alert={alertsSorted[index]} />
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
              <BreadcrumbPage>Alerts</BreadcrumbPage>
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

        <NavLink to="/alerts/create">
          <Button variant="outline">
            <Plus />
            Create Alert
          </Button>
        </NavLink>
      </header>

      <div className="flex flex-1 flex-col max-w-full overflow-hidden">
        <AutoSizer>
          {({ height, width }) => (
            <List
              itemData={alertsSorted} // this is to trigger a rerender on sort/filter
              height={height}
              width={width}
              itemSize={112} // 28rem, manually calculated
              itemCount={alertsSorted.length}
            >
              {ListItem}
            </List>
          )}
        </AutoSizer>
        {alerts.length === 0 && <NoAlerts className="m-auto" />}
        {alerts.length !== 0 && alertsSorted.length === 0 && (
          <NoAlertsFound className="m-auto" />
        )}
      </div>
    </>
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

function AlertListItem({ alert }: { alert: Alert }) {
  const app = useApp(alert.target.tag === "app" ? alert.target.id : null);
  const tag = useTag(alert.target.tag === "tag" ? alert.target.id : null);

  const currentUsage = useMemo(() => {
    const usages = alert.target.tag === "app" ? app?.usages : tag?.usages;
    switch (alert.timeFrame) {
      case "daily":
        return usages?.today;
      case "weekly":
        return usages?.week;
      case "monthly":
        return usages?.month;
    }
  }, [app, tag, alert]);

  // App's tag
  const appTag = useTag(app?.tagId ?? null);
  // Tag's list of Apps
  const tagApps = useApps(tag?.apps ?? []);

  return (
    <NavLink
      to={`/alerts/${alert.id}`}
      className={cn(
        "h-24 shadow-xs rounded-md flex flex-col",
        "ring-offset-background transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        "disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none",
        "bg-card text-card-foreground hover:bg-muted/75 border-border border",
      )}
    >
      <div className="h-20 flex items-center gap-2 p-4 @container">
        {alert.target.tag === "app" && app ? (
          <>
            <AppIcon buffer={app.icon} className="mx-2 h-10 w-10 shrink-0" />

            <div className="flex flex-col min-w-0">
              <div className="inline-flex items-center gap-2">
                <Text className="text-lg font-semibold max-w-72">
                  {app.name}
                </Text>
                {appTag && <MiniTagItem tag={appTag} />}
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
          </>
        ) : alert.target.tag === "tag" && tag ? (
          <>
            <TagIcon
              className="mx-2 h-10 w-10 shrink-0"
              style={{ color: tag.color }}
            />
            <div className="flex flex-col min-w-0 gap-1">
              <Text className="text-lg font-semibold max-w-72">{tag.name}</Text>
              {tagApps.length !== 0 && (
                <HorizontalOverflowList
                  className="gap-1 h-6"
                  items={tagApps}
                  renderItem={(app) => <MiniAppItem key={app.id} app={app} />}
                  renderOverflowItem={(app) => (
                    <MiniAppItem key={app.id} app={app} />
                  )}
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
          </>
        ) : null}
        <div className="flex-1" />

        <div className="flex flex-col items-end ml-auto py-2 ">
          <div className="text-sm flex gap-1 items-center">
            <span>
              {alert.triggerAction.tag === "dim"
                ? "Dim"
                : alert.triggerAction.tag === "message"
                  ? "Message"
                  : "Kill"}
            </span>
            {alert.triggerAction.tag === "dim" && (
              <div className="flex items-center">
                <span>(</span>
                <DurationText ticks={alert.triggerAction.duration} />
                <span>)</span>
              </div>
            )}
            {alert.triggerAction.tag === "message" && (
              <div className="flex items-center">
                <span>(</span>
                <Text className="max-w-24">{alert.triggerAction.content}</Text>
                <span>)</span>
              </div>
            )}
          </div>

          <div className="flex items-baseline text-card-foreground/50">
            <DurationText
              className="text-lg text-center text-card-foreground whitespace-nowrap"
              ticks={currentUsage ?? 0}
            />
            <span className="ml-2 mr-1">/</span>
            <DurationText
              className="text-center whitespace-nowrap"
              ticks={alert.usageLimit}
            />
            <span className="mr-1">,</span>
            <span>
              {alert.timeFrame === "daily"
                ? "Daily"
                : alert.timeFrame === "weekly"
                  ? "Weekly"
                  : "Monthly"}
            </span>
          </div>
        </div>
      </div>
      <div className="-mt-1 mx-2">
        <TimeProgressBar
          usageLimit={alert.usageLimit}
          currentUsage={currentUsage ?? 0}
          reminders={alert.reminders}
          circleRadius={7}
        />
      </div>
    </NavLink>
  );
}

function TimeProgressBar({
  className,
  usageLimit,
  currentUsage,
  reminders,
  circleRadius,
}: {
  className?: ClassValue;
  usageLimit: number;
  currentUsage: number;
  reminders: Reminder[];
  circleRadius: number;
}) {
  const percentage = (currentUsage / usageLimit) * 100;

  return (
    <div
      className={cn("relative w-full bg-secondary h-2 rounded-full", className)}
    >
      <div
        className="h-full bg-primary/90 rounded-full transition-all"
        style={{ width: `${Math.min(100, percentage)}%` }}
      />
      {reminders.map((reminder, index) => {
        return (
          <Tooltip key={index}>
            <TooltipTrigger asChild>
              <div
                className={cn(
                  "absolute top-1/2 border border-border rounded-full",
                  {
                    "bg-blue-500": reminder.status.tag == "hit",
                    "bg-gray-500": reminder.status.tag == "ignored",
                    "bg-blue-300": reminder.status.tag == "untriggered",
                  },
                )}
                style={{
                  left: `${reminder.threshold * 100}%`,
                  width: circleRadius * 2,
                  height: circleRadius * 2,
                  transform: `translate(-${circleRadius}px, -50%)`,
                }}
              />
            </TooltipTrigger>
            <TooltipContent>
              <p>{`${reminder.threshold} - ${reminder.message}`}</p>
            </TooltipContent>
          </Tooltip>
        );
      })}
    </div>
  );
}

import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import type { Alert } from "@/lib/entities";
import { Text } from "@/components/ui/text";
import { cn } from "@/lib/utils";
import AppIcon from "@/components/app/app-icon";
import { memo, useMemo, type CSSProperties, type ReactNode } from "react";
import { DurationText } from "@/components/time/duration-text";
import { useApps, useAlerts, useApp, useTag } from "@/hooks/use-refresh";
import { useAppState } from "@/lib/state";
import { NavLink } from "react-router";
import { Plus, TagIcon } from "lucide-react";
import AutoSizer from "react-virtualized-auto-sizer";
import { FixedSizeList as List } from "react-window";
import { HorizontalOverflowList } from "@/components/overflow-list";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { CreateAlertDialog } from "@/components/alert/create-alert-dialog";
import { MiniAppItem } from "@/routes/tags";
import { MiniTagItem } from "@/routes/apps";
import { useAlertsSearch } from "@/hooks/use-search";
import { SearchBar } from "@/components/search-bar";

export default function Alerts() {
  const alerts = useAlerts();
  const createAlert = useAppState((state) => state.createAlert);
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

        <div className="xl:flex-1 flex-none" />

        <SearchBar
          className="max-w-80 m-auto xl:m-0"
          placeholder="Search..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />

        <CreateAlertDialog
          trigger={
            <Button variant="outline">
              <Plus />
              Create Alert
            </Button>
          }
          onSubmit={async (alert) => {
            await createAlert(alert);
          }}
        ></CreateAlertDialog>
      </header>

      <div className="flex flex-1 flex-col max-w-full overflow-hidden">
        <AutoSizer>
          {({ height, width }) => (
            <List
              itemData={alertsSorted} // this is to trigger a rerender on sort/filter
              height={height}
              width={width}
              itemSize={96} // 24rem, manually calculated
              itemCount={alertsSorted.length}
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

function AlertListItem({ alert }: { alert: Alert }) {
  const app = useApp(alert.target.tag === "App" ? alert.target.id : null);
  const tag = useTag(alert.target.tag === "Tag" ? alert.target.id : null);

  const currentUsage = useMemo(() => {
    const usages = alert.target.tag === "App" ? app?.usages : tag?.usages;
    switch (alert.time_frame) {
      case "Daily":
        return usages?.today;
      case "Weekly":
        return usages?.week;
      case "Monthly":
        return usages?.month;
    }
  }, [app, tag, alert]);

  // App's tag
  const appTag = useTag(app?.tag_id ?? null);
  // Tag's list of Apps
  const tagApps = useApps(tag?.apps ?? []);

  return (
    <NavLink
      to={`/alerts/${alert.id}`}
      className={cn(
        "h-20 shadow-sm rounded-md flex items-center gap-2 p-4 @container",
        "ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        "disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none cursor-pointer",
        "bg-card text-card-foreground hover:bg-muted/75 border-border border",
      )}
    >
      {alert.target.tag === "App" && app ? (
        <>
          <AppIcon buffer={app.icon} className="mx-2 h-10 w-10 flex-shrink-0" />

          <div className="flex flex-col min-w-0">
            <div className="inline-flex items-center gap-2">
              <Text className="text-lg font-semibold max-w-72">{app.name}</Text>
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
      ) : alert.target.tag === "Tag" && tag ? (
        <>
          <TagIcon
            className="mx-2 h-10 w-10 flex-shrink-0"
            style={{ color: tag.color }}
          />
          <div className="flex flex-col min-w-0 gap-1">
            <Text className="text-lg font-semibold max-w-72">{tag.name}</Text>
            {tagApps.length !== 0 && (
              <HorizontalOverflowList
                className="gap-1 h-6 -mb-2"
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
          <span>{alert.trigger_action.tag}</span>
          {alert.trigger_action.tag === "Dim" && (
            <div className="flex items-center">
              <span>(</span>
              <DurationText ticks={alert.trigger_action.duration} />
              <span>)</span>
            </div>
          )}
          {alert.trigger_action.tag === "Message" && (
            <div className="flex items-center">
              <span>(</span>
              <Text className="max-w-24">{alert.trigger_action.content}</Text>
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
            ticks={alert.usage_limit}
          />
          <span className="mr-1">,</span>
          <span>{alert.time_frame}</span>
        </div>
      </div>
    </NavLink>
  );
}

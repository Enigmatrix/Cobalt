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
import { useMemo, type CSSProperties, type ReactNode } from "react";
import { Buffer } from "buffer";
import { CircleHelp } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { FixedSizeList as List } from "react-window";
import AutoSizer from "react-virtualized-auto-sizer";

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
  const icon = useMemo(
    () =>
      app.icon
        ? 'url("data:;base64,' + Buffer.from(app.icon).toString("base64") + '")'
        : undefined,
    [app.icon]
  );

  return (
    <div className="h-20 shadow-sm rounded-sm bg-muted flex overflow-auto">
      <div className="flex-1 flex items-center gap-2 p-4">
        {icon ? (
          <div
            className="mx-2 h-10 w-10 bg-no-repeat bg-center bg-cover"
            style={{ backgroundImage: icon }}
          />
        ) : (
          <CircleHelp className="mx-2 h-10 w-10" />
        )}

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
    </div>
  );
}

export default function Apps() {
  const apps = useAppState((state) => state.apps);
  const appsSorted = useMemo(() => {
    const appValues = Object.values(apps);
    // descending order of usage_today.
    appValues.sort(
      (app1, app2) => app2.usages.usage_today - app1.usages.usage_today
    );
    return appValues;
  }, [apps]);

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
      </header>

      <div className="flex flex-1 flex-col max-w-full overflow-hidden">
        <AutoSizer>
          {({ height, width }) => (
            <List
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

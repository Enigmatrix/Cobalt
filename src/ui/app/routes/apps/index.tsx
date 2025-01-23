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
import { useMemo } from "react";
import { Buffer } from "buffer";
import { CircleHelp } from "lucide-react";
import { Badge } from "@/components/ui/badge";

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
      className="truncate max-w-16"
    >
      {tag.name}
    </Badge>
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
    <div className="h-24 shadow-sm rounded-sm bg-muted flex overflow-auto">
      <div className="flex-1 flex items-center gap-2 p-4">
        {icon ? (
          <div
            className="mx-2 h-12 w-12 bg-no-repeat bg-center bg-cover"
            style={{ backgroundImage: icon }}
          />
        ) : (
          <CircleHelp className="mx-2 h-12 w-12 " />
        )}

        <div className="flex flex-col">
          <div className="text-lg font-semibold max-w-72 truncate">
            {app.name}
          </div>
          <span className="inline-flex gap-1 items-center text-white/50 text-sm">
            <p className="max-w-48 truncate">{app.company}</p>
            {app.description && (
              <>
                <p>|</p>
                <p className=" max-w-[40rem] truncate">{app.description}</p>
              </>
            )}
          </span>
          {/* TODO show horizontal scrolling on too many tags */}
          <div>
            {app.tags.map((tagId) => (
              <TagItem key={tagId} tagId={tagId} />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default function Apps() {
  const apps = useAppState((state) => state.apps);

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

      <div className="flex flex-1 flex-col gap-4 p-4">
        {Object.values(apps).map((app) => (
          <AppListItem key={app.id} app={app} />
        ))}
      </div>
    </>
  );
}

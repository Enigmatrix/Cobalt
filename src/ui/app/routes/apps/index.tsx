import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import { useAppState } from "@/lib/state";
import type { App } from "@/lib/entities";
import { useMemo } from "react";
import { Buffer } from "buffer";
import { CircleHelp } from "lucide-react";

function AppListItem({ app }: { app: App }) {
  const icon = useMemo(
    () =>
      app.icon
        ? 'url("data:;base64,' + Buffer.from(app.icon).toString("base64") + '")'
        : undefined,
    [app.icon]
  );

  return (
    <div className="h-24 shadow-sm rounded-sm bg-muted flex ">
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
          <div className="inline-flex place-items-baseline gap-2">
            <div className="text-lg font-semibold max-w-72 truncate">
              {app.name}
            </div>
            <div>-</div>
            <div className="text-sm max-w-48 truncate text-white/75">
              {app.company}
            </div>
          </div>
          <div className="text-sm max-w-[40rem] truncate text-white/50">
            {app.description}
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

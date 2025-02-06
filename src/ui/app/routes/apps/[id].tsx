import { SidebarTrigger } from "@/components/ui/sidebar";
import type { Route } from "../apps/+types/[id]";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
  BreadcrumbLink,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { useAppState } from "@/lib/state";
import type { App, Ref } from "@/lib/entities";
import AppIcon from "@/components/app-icon";
import { toHumanDuration } from "@/lib/time";

function CardUsage({
  title,
  usage,
}: {
  title: string;
  usage: number;
  totalUsage: number;
}) {
  return (
    <div className="aspect-video rounded-xl bg-muted/50 flex">
      <div className="flex flex-col items-center m-auto">
        <div className="text-sm md:text-base lg:text-lg text-primary/50">
          {title}
        </div>
        <div className="text-2xl md:text-3xl lg:text-5xl font-semibold tracking-tighter">
          {toHumanDuration(usage)}
        </div>
      </div>
    </div>
  );
}

export default function App({ params }: Route.ComponentProps) {
  const id = +params.id;
  const app = useAppState((state) => state.apps[id as Ref<App>])!;
  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbLink href="/apps">Apps</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem>
              <BreadcrumbPage className="inline-flex items-center">
                <AppIcon buffer={app.icon} className="w-5 h-5 mr-2" />
                {app.name}
              </BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>
      <div className="flex flex-1 flex-col gap-4 p-4">
        <div className="grid auto-rows-min gap-4 md:grid-cols-3">
          <CardUsage
            title="Today"
            usage={app.usages.usage_today}
            totalUsage={0}
          />
          <CardUsage
            title="Week"
            usage={app.usages.usage_week}
            totalUsage={0}
          />
          <CardUsage
            title="Month"
            usage={app.usages.usage_month}
            totalUsage={0}
          />
        </div>
        <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min" />
      </div>
    </>
  );
}

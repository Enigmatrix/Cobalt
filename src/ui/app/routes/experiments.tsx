import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import { refresh, useAppState } from "@/lib/state";
import { getAppDurationsPerPeriod } from "@/lib/repo";
import { dateTimeToTicks, durationToTicks } from "@/lib/time";
import { DateTime, Duration } from "luxon";
import { useEffect, useState } from "react";
import _ from "lodash";
import type { App, Ref, WithGroupedDuration } from "@/lib/entities";
import { AppUsageBarChart } from "@/components/app-usage-chart";

async function copySeedDb() {
  await invoke("copy_seed_db");
}

async function updateUsagesEnd() {
  await invoke("update_usages_end");
}

async function refreshState() {
  await refresh();
}

export default function Experiments() {
  const [data, setData] = useState<WithGroupedDuration<App>[]>([]);
  const [data6, setData6] = useState<WithGroupedDuration<App>[]>([]);
  const apps = useAppState((state) => state.apps);
  const chrome = apps[6 as unknown as Ref<App>] as App;
  const now = DateTime.now().startOf("day").plus({ day: 1 });

  const periodTicks = durationToTicks(Duration.fromDurationLike({ day: 1 }));
  const minTicks = dateTimeToTicks(now.minus({ week: 1 }));
  const maxTicks = dateTimeToTicks(now);
  console.log("minTicks", minTicks, "maxTicks", maxTicks);
  useEffect(() => {
    getAppDurationsPerPeriod({
      period: periodTicks,
      start: minTicks,
      end: maxTicks,
    }).then((apps) => {
      setData(_(apps).values().flatten().value());
      setData6(_(apps[chrome.id]).flatten().value());
    });
  }, [periodTicks, minTicks, maxTicks]);
  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbPage>Experiments</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>
      <div className="flex flex-1 flex-col gap-4 p-4">
        <div className="grid auto-rows-min gap-4 md:grid-cols-3">
          <div className="aspect-video rounded-xl bg-muted/50">
            <Button onClick={copySeedDb} variant="outline">
              Copy seed.db
            </Button>
            <Button onClick={updateUsagesEnd} variant="outline">
              Set last usage to now
            </Button>
            <Button onClick={refreshState} variant="outline">
              Refresh
            </Button>
          </div>
          <div className="aspect-video rounded-xl bg-muted/50" />
          <div className="aspect-video rounded-xl bg-muted/50" />
        </div>

        <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min flex flex-col gap-4 p-4">
          <AppUsageBarChart
            periodTicks={periodTicks}
            data={data}
            rangeMinTicks={minTicks}
            rangeMaxTicks={maxTicks}
            onHover={() => console.log("")}
          />
        </div>
        <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min flex flex-col gap-4 p-4">
          <AppUsageBarChart
            periodTicks={periodTicks}
            data={data6}
            singleAppId={6 as unknown as Ref<App>}
            onHover={() => console.log("")}
          />
        </div>
      </div>
    </>
  );
}

import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import { Button, type ButtonProps } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import { refresh } from "@/lib/state";
import { useTransition } from "react";
import { LoaderIcon } from "lucide-react";

async function copySeedDb() {
  await invoke("copy_seed_db");
}

async function updateUsagesEnd() {
  await invoke("update_usages_end");
}

async function refreshState() {
  await refresh();
}

function AsyncButton({
  onClick,
  children,
  ...props
}: Omit<ButtonProps, "onClick"> & { onClick: () => Promise<void> }) {
  const [isLoading, startTransition] = useTransition();

  return (
    <Button
      {...props}
      children={isLoading ? <LoaderIcon className="animate-spin" /> : children}
      onClick={() => startTransition(async () => await onClick())}
    />
  );
}

export default function Experiments() {
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
          <div className="aspect-video rounded-xl bg-muted/50" />
          <div className="aspect-video rounded-xl bg-muted/50" />
          <div className="aspect-video rounded-xl bg-muted/50" />
        </div>
        <div className="min-h-[100vh] flex-1 rounded-xl bg-muted/50 md:min-h-min flex flex-col gap-4 p-4">
          <AsyncButton onClick={copySeedDb} variant="outline">
            Copy seed.db
          </AsyncButton>
          <AsyncButton onClick={updateUsagesEnd} variant="outline">
            Set last usage to now
          </AsyncButton>
          <AsyncButton onClick={refreshState} variant="outline">
            Refresh
          </AsyncButton>
        </div>
      </div>
    </>
  );
}

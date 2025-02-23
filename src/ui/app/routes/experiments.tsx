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
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Setting } from "@/routes/settings";

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
        <Card>
          <CardHeader>
            <CardTitle>Database</CardTitle>
          </CardHeader>
          <CardContent className="gap-2 flex flex-col">
            <Setting
              title={
                <>
                  Copy <p className="font-mono inline">seed.db</p>
                </>
              }
              description="Replace the current database with the seed database"
              action={
                <AsyncButton onClick={copySeedDb} variant="outline">
                  Run
                </AsyncButton>
              }
            />

            <Setting
              title="Update Last Usage to Now"
              description="Update last usage in database to Now"
              action={
                <AsyncButton onClick={updateUsagesEnd} variant="outline">
                  Run
                </AsyncButton>
              }
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Refresh</CardTitle>
          </CardHeader>
          <CardContent className="gap-2 flex flex-col">
            <Setting
              title="Refresh Now"
              description="Refresh all app data"
              action={
                <AsyncButton onClick={refreshState} variant="outline">
                  Run
                </AsyncButton>
              }
            />
          </CardContent>
        </Card>
      </div>
    </>
  );
}

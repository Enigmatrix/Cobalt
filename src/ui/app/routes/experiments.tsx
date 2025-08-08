import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { refresh } from "@/lib/state";
import {
  Setting,
  SettingContent,
  SettingHeader,
  SettingItem,
  SettingTitle,
} from "@/routes/settings";
import { invoke } from "@tauri-apps/api/core";
import { LoaderIcon } from "lucide-react";
import { useTransition, type ComponentProps } from "react";

async function copySeedDb() {
  await invoke("copy_from_seed_db");
}

async function copyInstallDb() {
  await invoke("copy_from_install_db");
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
}: Omit<ComponentProps<typeof Button>, "onClick"> & {
  onClick: () => Promise<void>;
}) {
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
      <div className="h-0 flex-auto overflow-auto [scrollbar-gutter:stable]">
        <div className="flex flex-col gap-4 p-4">
          <Setting>
            <SettingHeader>
              <SettingTitle>Database</SettingTitle>
            </SettingHeader>
            <SettingContent className="gap-2 flex flex-col">
              <SettingItem
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

              <SettingItem
                title={<>Copy install db</>}
                description="Replace the current database with the installed database"
                action={
                  <AsyncButton onClick={copyInstallDb} variant="outline">
                    Run
                  </AsyncButton>
                }
              />

              <SettingItem
                title="Update Last Usage to Now"
                description="Update last usage in database to Now"
                action={
                  <AsyncButton onClick={updateUsagesEnd} variant="outline">
                    Run
                  </AsyncButton>
                }
              />
            </SettingContent>
          </Setting>

          <Setting>
            <SettingHeader>
              <SettingTitle>Refresh</SettingTitle>
            </SettingHeader>
            <SettingContent className="gap-2 flex flex-col">
              <SettingItem
                title="Refresh Now"
                description="Refresh all app data"
                action={
                  <AsyncButton onClick={refreshState} variant="outline">
                    Run
                  </AsyncButton>
                }
              />
            </SettingContent>
          </Setting>
        </div>
      </div>
    </>
  );
}

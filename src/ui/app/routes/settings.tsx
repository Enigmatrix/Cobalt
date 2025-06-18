import { SidebarTrigger } from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbList,
} from "@/components/ui/breadcrumb";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ThemeSwitch } from "@/components/theme-switch";
import { useTheme } from "@/components/theme-provider";
import { useEffect, useState, type ReactNode } from "react";
import { Switch } from "@/components/ui/switch";
import {
  readConfig,
  setTrackIncognito as setTrackIncognitoFn,
} from "@/lib/config";
import { error } from "@/lib/log";

export function Setting({
  title,
  description,
  action,
}: {
  title: ReactNode;
  description: ReactNode;
  action: ReactNode;
}) {
  return (
    <div className="flex items-center">
      <div>
        <h3 className="text-lg font-semibold text-card-foreground/80">
          {title}
        </h3>
        <p className="text-sm text-card-foreground/50">{description}</p>
      </div>

      <div className="flex-1"></div>

      {action}
    </div>
  );
}

export default function Settings() {
  const { theme, setTheme } = useTheme();
  const [trackIncognito, setTrackIncognitoInner] = useState(false);

  useEffect(() => {
    readConfig()
      .then((config) => {
        setTrackIncognitoInner(config.trackIncognito);
      })
      .catch((err) => {
        error("Failed to read config", err);
      });
  }, []);

  function setTrackIncognito(value: boolean) {
    setTrackIncognitoFn(value)
      .then(() => {
        setTrackIncognitoInner(value);
      })
      .catch((err) => {
        error("Failed to set track incognito", err);
      });
  }

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbPage>Settings</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>
      <div className="h-0 flex-auto overflow-auto [scrollbar-gutter:stable]">
        <div className="flex flex-col gap-4 p-4">
          <Card>
            <CardHeader>
              <CardTitle>Appearance</CardTitle>
            </CardHeader>
            <CardContent>
              <Setting
                title="Theme"
                description="Choose a theme for the app"
                action={<ThemeSwitch value={theme} onValueChange={setTheme} />}
              />
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Privacy</CardTitle>
            </CardHeader>
            <CardContent>
              <Setting
                title="Track Incognito"
                description="Track incognito windows (title, url) in browsers. If disabled, incognito windows will be tracked as '<Incognito>'."
                action={
                  <Switch
                    checked={trackIncognito}
                    onCheckedChange={setTrackIncognito}
                  />
                }
              />
            </CardContent>
          </Card>
        </div>
      </div>
    </>
  );
}

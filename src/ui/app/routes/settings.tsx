import { ScoreSlider } from "@/components/tag/score-slider";
import { useTheme } from "@/components/theme-provider";
import { ThemeSwitch } from "@/components/theme-switch";
import { DurationPicker } from "@/components/time/duration-picker";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { Switch } from "@/components/ui/switch";
import { useConfig } from "@/lib/config";
import type { Score } from "@/lib/entities";
import { durationToTicks, ticksToDuration } from "@/lib/time";
import { RefreshCcwIcon } from "lucide-react";
import { type ReactNode } from "react";

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
    <div className="flex items-center gap-2">
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
  const {
    trackIncognito,
    defaultFocusStreakSettings,
    defaultDistractiveStreakSettings,
    setTrackIncognito,
    setDefaultFocusStreakSettings,
    setDefaultDistractiveStreakSettings,
    resetDefaultFocusStreakSettings,
    resetDefaultDistractiveStreakSettings,
  } = useConfig();

  const resetStreakSettings = async () => {
    await resetDefaultFocusStreakSettings();
    await resetDefaultDistractiveStreakSettings();
  };

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
          <Card>
            <CardHeader>
              <CardTitle>
                <div className="flex items-center">
                  <div>Streaks</div>
                  <Button
                    variant="outline"
                    onClick={resetStreakSettings}
                    className="ml-auto"
                  >
                    <RefreshCcwIcon className="w-4 h-4 text-muted-foreground" />
                    <div className="text-muted-foreground">Reset</div>
                  </Button>
                </div>
              </CardTitle>
              <CardDescription>Settings for streak detection.</CardDescription>
            </CardHeader>
            <CardContent className="gap-4 flex flex-col">
              <Setting
                title="Focus: Minimum Score"
                description="The minimum score of an app's tag for the app to be considered a focus app."
                action={
                  <ScoreSettingAction
                    score={defaultFocusStreakSettings.minFocusScore}
                    onScoreChange={(v) =>
                      setDefaultFocusStreakSettings({ minFocusScore: v })
                    }
                  />
                }
              />

              <Setting
                title="Focus: Minimum Usage Duration"
                description="The minimum duration of contiguous usages from focus apps to be considered a focus streak."
                action={
                  <DurationPicker
                    className="w-40"
                    value={ticksToDuration(
                      defaultFocusStreakSettings.minFocusUsageDur,
                    )}
                    onValueChange={(v) =>
                      v !== null &&
                      setDefaultFocusStreakSettings({
                        minFocusUsageDur: durationToTicks(v),
                      })
                    }
                  />
                }
              />

              <Setting
                title="Focus: Maximum Gap"
                description="The maximum gap between focus streaks before adjacent focus streaks are combined into a longer streak."
                action={
                  <DurationPicker
                    className="w-40"
                    value={ticksToDuration(
                      defaultFocusStreakSettings.maxFocusGap,
                    )}
                    onValueChange={(v) =>
                      v !== null &&
                      setDefaultFocusStreakSettings({
                        maxFocusGap: durationToTicks(v),
                      })
                    }
                  />
                }
              />

              <Setting
                title="Distractive: Maximum Score"
                description="The maximum score of an app's tag for the app to be considered a distractive app."
                action={
                  <ScoreSettingAction
                    score={defaultDistractiveStreakSettings.maxDistractiveScore}
                    onScoreChange={(v) =>
                      setDefaultDistractiveStreakSettings({
                        maxDistractiveScore: v,
                      })
                    }
                  />
                }
              />

              <Setting
                title="Distractive: Minimum Usage Duration"
                description="The minimum duration of contiguous usages from distractive apps to be considered a distractive streak."
                action={
                  <DurationPicker
                    className="w-40"
                    value={ticksToDuration(
                      defaultDistractiveStreakSettings.minDistractiveUsageDur,
                    )}
                    onValueChange={(v) =>
                      v !== null &&
                      setDefaultDistractiveStreakSettings({
                        minDistractiveUsageDur: durationToTicks(v),
                      })
                    }
                  />
                }
              />

              <Setting
                title="Distractive: Maximum Gap"
                description="The maximum gap between distractive streaks before adjacent distractive streaks are combined into a longer streak."
                action={
                  <DurationPicker
                    className="w-40"
                    value={ticksToDuration(
                      defaultDistractiveStreakSettings.maxDistractiveGap,
                    )}
                    onValueChange={(v) =>
                      v !== null &&
                      setDefaultDistractiveStreakSettings({
                        maxDistractiveGap: durationToTicks(v),
                      })
                    }
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

function ScoreSettingAction({
  score,
  onScoreChange,
}: {
  score: Score;
  // TODO: we should add some debouncing here otherwise we get too much file io
  onScoreChange: (score: Score) => void;
}) {
  return (
    <div className="flex flex-col items-center">
      <div className="h-8 mb-1 text-sm gap-2 flex items-center">
        <span className="text-sm text-muted-foreground min-w-0 truncate">
          {score}
        </span>
        {score !== 0 && (
          <Button
            variant="outline"
            size="sm"
            className="px-2 py-1 h-6 m-0 text-xs"
            onClick={() => onScoreChange(0)}
          >
            Reset
          </Button>
        )}
      </div>

      <ScoreSlider
        className="w-40"
        value={score}
        onValueChange={onScoreChange}
      />
    </div>
  );
}

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
import { useDebouncedState } from "@/hooks/use-debounced-state";
import { useConfig } from "@/lib/config";
import type { Score } from "@/lib/entities";
import { durationToTicks, ticksToDuration } from "@/lib/time";
import { cn } from "@/lib/utils";
import { RefreshCcwIcon } from "lucide-react";
import { useCallback, type ReactNode } from "react";

export default function Settings() {
  const { theme, setTheme } = useTheme();
  const { trackIncognito, setTrackIncognito } = useConfig();

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
          <Setting>
            <SettingHeader>
              <SettingTitle>Appearance</SettingTitle>
            </SettingHeader>
            <SettingContent>
              <SettingItem
                title="Theme"
                description="Choose a theme for the app"
                action={<ThemeSwitch value={theme} onValueChange={setTheme} />}
              />
            </SettingContent>
          </Setting>
          <Setting>
            <SettingHeader>
              <SettingTitle>Privacy</SettingTitle>
            </SettingHeader>
            <SettingContent>
              <SettingItem
                title="Track Incognito"
                description="Track incognito windows (title, url) in browsers. If disabled, incognito windows will be tracked as '<Incognito>'."
                action={
                  <Switch
                    checked={trackIncognito}
                    onCheckedChange={setTrackIncognito}
                  />
                }
              />
            </SettingContent>
          </Setting>

          <FocusStreakSetting />

          <DistractiveStreakSetting />
        </div>
      </div>
    </>
  );
}

function FocusStreakSetting() {
  const {
    defaultFocusStreakSettings,
    setDefaultFocusStreakSettings,
    resetDefaultFocusStreakSettings,
  } = useConfig();

  const [minFocusScore, setMinFocusScore, setMinFocusScoreInner] =
    useDebouncedState(
      defaultFocusStreakSettings.minFocusScore,
      async (v) => {
        await setDefaultFocusStreakSettings({ minFocusScore: v });
      },
      500,
    );

  const reset = useCallback(async () => {
    const config = await resetDefaultFocusStreakSettings();
    // Reset the min focus score to the default value
    setMinFocusScoreInner(config.defaultFocusStreakSettings.minFocusScore);
  }, [resetDefaultFocusStreakSettings, setMinFocusScoreInner]);

  return (
    <Setting>
      <SettingHeader>
        <SettingTitle>
          <div className="flex items-center">
            <div>Focus Streaks</div>
            <Button variant="outline" onClick={reset} className="ml-auto">
              <RefreshCcwIcon className="w-4 h-4 text-muted-foreground" />
              <div className="text-muted-foreground">Reset</div>
            </Button>
          </div>
        </SettingTitle>
        <SettingDescription>
          Settings for focus streak detection.
        </SettingDescription>
      </SettingHeader>
      <SettingContent className="gap-4 flex flex-col">
        <SettingItem
          title="Minimum Score"
          description="The minimum score of an app's tag for the app to be considered a focus app."
          action={
            <ScoreSettingAction
              score={minFocusScore}
              onScoreChange={setMinFocusScore}
            />
          }
        />

        <SettingItem
          title="Minimum Usage Duration"
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

        <SettingItem
          title="Maximum Gap"
          description="The maximum gap between focus streaks before adjacent focus streaks are combined into a longer streak."
          action={
            <DurationPicker
              className="w-40"
              value={ticksToDuration(defaultFocusStreakSettings.maxFocusGap)}
              onValueChange={(v) =>
                v !== null &&
                setDefaultFocusStreakSettings({
                  maxFocusGap: durationToTicks(v),
                })
              }
            />
          }
        />
      </SettingContent>
    </Setting>
  );
}

function DistractiveStreakSetting() {
  const {
    defaultDistractiveStreakSettings,
    setDefaultDistractiveStreakSettings,
    resetDefaultDistractiveStreakSettings,
  } = useConfig();

  const [
    maxDistractiveScore,
    setMaxDistractiveScore,
    setMaxDistractiveScoreInner,
  ] = useDebouncedState(
    defaultDistractiveStreakSettings.maxDistractiveScore,
    async (v) => {
      await setDefaultDistractiveStreakSettings({ maxDistractiveScore: v });
    },
    500,
  );

  const reset = useCallback(async () => {
    const config = await resetDefaultDistractiveStreakSettings();
    // Reset the max distractive score to the default value
    setMaxDistractiveScoreInner(
      config.defaultDistractiveStreakSettings.maxDistractiveScore,
    );
  }, [resetDefaultDistractiveStreakSettings, setMaxDistractiveScoreInner]);

  return (
    <Setting>
      <SettingHeader>
        <SettingTitle>
          <div className="flex items-center">
            <div>Distractive Streaks</div>
            <Button variant="outline" onClick={reset} className="ml-auto">
              <RefreshCcwIcon className="w-4 h-4 text-muted-foreground" />
              <div className="text-muted-foreground">Reset</div>
            </Button>
          </div>
        </SettingTitle>
        <SettingDescription>
          Settings for distractive streak detection.
        </SettingDescription>
      </SettingHeader>
      <SettingContent className="gap-4 flex flex-col">
        <SettingItem
          title="Maximum Score"
          description="The maximum score of an app's tag for the app to be considered a distractive app."
          action={
            <ScoreSettingAction
              score={maxDistractiveScore}
              onScoreChange={setMaxDistractiveScore}
            />
          }
        />

        <SettingItem
          title="Minimum Usage Duration"
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

        <SettingItem
          title="Maximum Gap"
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
      </SettingContent>
    </Setting>
  );
}

function ScoreSettingAction({
  score,
  onScoreChange,
}: {
  score: Score;
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

export function Setting(props: React.ComponentProps<typeof Card>) {
  return <Card {...props} />;
}

export function SettingHeader({
  className,
  ...props
}: React.ComponentProps<typeof CardHeader>) {
  return <CardHeader className={cn("gap-0", className)} {...props} />;
}

export function SettingDescription(
  props: React.ComponentProps<typeof CardDescription>,
) {
  return <CardDescription {...props} />;
}

export function SettingContent(
  props: React.ComponentProps<typeof CardContent>,
) {
  return <CardContent {...props} />;
}

export function SettingTitle({
  className,
  ...props
}: React.ComponentProps<typeof CardTitle>) {
  return <CardTitle className={cn("text-lg", className)} {...props} />;
}

export function SettingItem({
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
        <h3 className="font-semibold text-card-foreground/80">{title}</h3>
        <p className="text-card-foreground/50">{description}</p>
      </div>

      <div className="flex-1"></div>

      {action}
    </div>
  );
}

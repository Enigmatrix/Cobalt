import type {
  DistractiveStreakSettings,
  FocusStreakSettings,
} from "@/lib/entities";
import { invoke } from "@tauri-apps/api/core";
import { Duration } from "luxon";
import { create } from "zustand";
import { durationToTicks, ticksToDuration } from "./time";

export interface ConfigDuration {
  secs: number;
  nanos: number;
}

export interface RawConfig {
  trackIncognito?: boolean;
  maxIdleDuration: ConfigDuration;
  pollDuration: ConfigDuration;
  alertDuration: ConfigDuration;
  defaultFocusStreakSettings: RawFocusStreakSettings;
  defaultDistractiveStreakSettings: RawDistractiveStreakSettings;
}

export interface RawFocusStreakSettings {
  minFocusScore: number;
  minFocusUsageDur: ConfigDuration;
  maxFocusGap: ConfigDuration;
}

export interface RawDistractiveStreakSettings {
  maxDistractiveScore: number;
  minDistractiveUsageDur: ConfigDuration;
  maxDistractiveGap: ConfigDuration;
}

export function toDuration(duration: ConfigDuration) {
  return Duration.fromObject({
    seconds: duration.secs,
    milliseconds: duration.nanos / 1000000,
  });
}

export function fromDuration(duration: Duration) {
  const millis = duration.toMillis();
  return {
    secs: Math.floor(millis / 1000),
    nanos: Math.floor((millis % 1000) * 1000000),
  };
}

interface ConfigReadonly {
  trackIncognito: boolean;

  defaultFocusStreakSettings: FocusStreakSettings;
  defaultDistractiveStreakSettings: DistractiveStreakSettings;

  maxIdleDuration: Duration;
  pollDuration: Duration;
  alertDuration: Duration;
}

interface Config extends ConfigReadonly {
  setTrackIncognito: (value: boolean) => Promise<void>;
  setDefaultFocusStreakSettings: (
    value: Partial<FocusStreakSettings>,
  ) => Promise<void>;
  setDefaultDistractiveStreakSettings: (
    value: Partial<DistractiveStreakSettings>,
  ) => Promise<void>;
  resetDefaultFocusStreakSettings: () => Promise<void>;
  resetDefaultDistractiveStreakSettings: () => Promise<void>;
}

export const useConfig = create<Config>((set) => {
  return {
    trackIncognito: false,

    defaultFocusStreakSettings: {
      minFocusScore: 0,
      minFocusUsageDur: durationToTicks(toDuration({ secs: 0, nanos: 0 })),
      maxFocusGap: durationToTicks(toDuration({ secs: 0, nanos: 0 })),
    },
    defaultDistractiveStreakSettings: {
      maxDistractiveScore: 0,
      minDistractiveUsageDur: durationToTicks(
        toDuration({ secs: 0, nanos: 0 }),
      ),
      maxDistractiveGap: durationToTicks(toDuration({ secs: 0, nanos: 0 })),
    },

    maxIdleDuration: Duration.fromObject({ seconds: 0 }),
    pollDuration: Duration.fromObject({ seconds: 0 }),
    alertDuration: Duration.fromObject({ seconds: 0 }),

    setTrackIncognito: async (value) => {
      await setTrackIncognito(value);
      set(await readConfigReadonly());
    },
    setDefaultFocusStreakSettings: async (value) => {
      // Assumes FocusStreakSettings is shallow-cloneable
      const newValue = Object.assign(
        {},
        useConfig.getState().defaultFocusStreakSettings,
        value,
      );
      await setDefaultFocusStreakSettings({
        minFocusScore: newValue.minFocusScore,
        minFocusUsageDur: fromDuration(
          ticksToDuration(newValue.minFocusUsageDur),
        ),
        maxFocusGap: fromDuration(ticksToDuration(newValue.maxFocusGap)),
      });
      set(await readConfigReadonly());
    },
    setDefaultDistractiveStreakSettings: async (value) => {
      // Assumes DistractiveStreakSettings is shallow-cloneable
      const newValue = Object.assign(
        {},
        useConfig.getState().defaultDistractiveStreakSettings,
        value,
      );
      await setDefaultDistractiveStreakSettings({
        maxDistractiveScore: newValue.maxDistractiveScore,
        minDistractiveUsageDur: fromDuration(
          ticksToDuration(newValue.minDistractiveUsageDur),
        ),
        maxDistractiveGap: fromDuration(
          ticksToDuration(newValue.maxDistractiveGap),
        ),
      });
      set(await readConfigReadonly());
    },
    resetDefaultFocusStreakSettings: async () => {
      await resetDefaultFocusStreakSettings();
      set(await readConfigReadonly());
    },
    resetDefaultDistractiveStreakSettings: async () => {
      await resetDefaultDistractiveStreakSettings();
      set(await readConfigReadonly());
    },
  };
});

export async function refresh() {
  const config = await readConfigReadonly();
  useConfig.setState(config);
}

export async function readConfigReadonly() {
  const raw = await invoke<RawConfig>("read_config");
  return {
    trackIncognito: raw.trackIncognito ?? false,

    defaultFocusStreakSettings: {
      minFocusScore: raw.defaultFocusStreakSettings.minFocusScore,
      minFocusUsageDur: durationToTicks(
        toDuration(raw.defaultFocusStreakSettings.minFocusUsageDur),
      ),
      maxFocusGap: durationToTicks(
        toDuration(raw.defaultFocusStreakSettings.maxFocusGap),
      ),
    },
    defaultDistractiveStreakSettings: {
      maxDistractiveScore:
        raw.defaultDistractiveStreakSettings.maxDistractiveScore,
      minDistractiveUsageDur: durationToTicks(
        toDuration(raw.defaultDistractiveStreakSettings.minDistractiveUsageDur),
      ),
      maxDistractiveGap: durationToTicks(
        toDuration(raw.defaultDistractiveStreakSettings.maxDistractiveGap),
      ),
    },

    maxIdleDuration: toDuration(raw.maxIdleDuration),
    pollDuration: toDuration(raw.pollDuration),
    alertDuration: toDuration(raw.alertDuration),
  } satisfies ConfigReadonly;
}

async function setTrackIncognito(value: boolean) {
  return await invoke("config_set_track_incognito", { value });
}

async function setDefaultFocusStreakSettings(value: RawFocusStreakSettings) {
  return await invoke("config_set_default_focus_streak_settings", { value });
}

async function setDefaultDistractiveStreakSettings(
  value: RawDistractiveStreakSettings,
) {
  return await invoke("config_set_default_distractive_streak_settings", {
    value,
  });
}

async function resetDefaultFocusStreakSettings() {
  return await invoke("config_reset_default_focus_streak_settings");
}

async function resetDefaultDistractiveStreakSettings() {
  return await invoke("config_reset_default_distractive_streak_settings");
}

export async function getIconsDir(): Promise<string> {
  return await invoke("get_icons_dir");
}

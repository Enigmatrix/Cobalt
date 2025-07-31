import { invoke } from "@tauri-apps/api/core";
import { Duration } from "luxon";
import { create } from "zustand";

export interface ConfigDuration {
  secs: number;
  nanos: number;
}

export interface RawConfig {
  trackIncognito?: boolean;
  maxIdleDuration: ConfigDuration;
  pollDuration: ConfigDuration;
  alertDuration: ConfigDuration;
}

export function toDuration(duration: ConfigDuration) {
  return Duration.fromObject({
    seconds: duration.secs,
    milliseconds: duration.nanos / 1000000,
  });
}

interface ConfigReadonly {
  trackIncognito: boolean;

  maxIdleDuration: Duration;
  pollDuration: Duration;
  alertDuration: Duration;
}

interface Config extends ConfigReadonly {
  setTrackIncognito: (value: boolean) => Promise<void>;
}

export const useConfig = create<Config>((set) => {
  return {
    trackIncognito: false,
    maxIdleDuration: Duration.fromObject({ seconds: 0 }),
    pollDuration: Duration.fromObject({ seconds: 0 }),
    alertDuration: Duration.fromObject({ seconds: 0 }),
    setTrackIncognito: async (value) => {
      await setTrackIncognito(value);
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
    maxIdleDuration: toDuration(raw.maxIdleDuration),
    pollDuration: toDuration(raw.pollDuration),
    alertDuration: toDuration(raw.alertDuration),
  } satisfies ConfigReadonly;
}

async function setTrackIncognito(value: boolean) {
  return await invoke("config_set_track_incognito", { value });
}

export async function getIconsDir(): Promise<string> {
  return await invoke("get_icons_dir");
}

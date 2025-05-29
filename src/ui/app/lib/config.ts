import { invoke } from "@tauri-apps/api/core";
import { Duration } from "luxon";

export type ConfigDuration = {
  secs: number;
  nanos: number;
};

export type RawConfig = {
  trackIncognito?: boolean;
  maxIdleDuration: ConfigDuration;
  pollDuration: ConfigDuration;
  alertDuration: ConfigDuration;
};

export function toDuration(duration: ConfigDuration) {
  return Duration.fromObject({
    seconds: duration.secs,
    milliseconds: duration.nanos / 1000000,
  });
}

export class Config {
  private raw: RawConfig;

  constructor(raw: RawConfig) {
    this.raw = raw;
  }

  get trackIncognito() {
    return this.raw.trackIncognito ?? false;
  }

  get maxIdleDuration() {
    return toDuration(this.raw.maxIdleDuration);
  }

  get pollDuration() {
    return toDuration(this.raw.pollDuration);
  }

  get alertDuration() {
    return toDuration(this.raw.alertDuration);
  }
}

export async function readConfig() {
  return new Config(await invoke<RawConfig>("read_config"));
}

export async function setTrackIncognito(value: boolean) {
  return await invoke("config_set_track_incognito", { value });
}

import humanizeDuration from "pretty-ms";
import { DateTime } from "luxon";

export function toHumanDuration(ticks: number): string {
  return humanizeDuration(ticks / 10_000, { unitCount: 2 });
}

export function toTicks(ts: DateTime): number {
  return (ts.toMillis() + 62_135_596_800_000) * 10_000;
}

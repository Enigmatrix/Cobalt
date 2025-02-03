import humanizeDuration from "pretty-ms";
import { DateTime, Duration } from "luxon";

export function toHumanDuration(
  ticks: number,
  showSymbolForZero: boolean = true,
  symbolForZero: string = "-"
): string {
  return ticks === 0 && showSymbolForZero
    ? symbolForZero
    : humanizeDuration(ticks / 10_000, { unitCount: 2 });
}

export function dateTimeToTicks(ts: DateTime): number {
  return (ts.toMillis() + 62_135_596_800_000) * 10_000;
}

export function durationToTicks(ts: Duration): number {
  return ts.toMillis() * 10_000;
}

export function ticksToDuration(ts: number): Duration {
  return Duration.fromMillis(ts / 10_000);
}

export function ticksToDateTime(ticks: number): DateTime {
  return DateTime.fromMillis(ticks / 10_000 - 62_135_596_800_000);
}

export function toHumanDateTime(dt: DateTime): string {
  // if time part is 00:00:00, then return date part only
  if (dt.toFormat("HH:mm:ss") === "00:00:00") {
    return dt.toFormat("LLL dd, y");
  }
  if (dt.toFormat("ss") === "00") {
    return dt.toFormat("LLL dd, y hh:mm a");
  }
  return dt.toFormat("LLL dd, y hh:mm:ss a");
}

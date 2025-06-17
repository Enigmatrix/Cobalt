import humanizeDuration from "pretty-ms";
import { DateTime, Duration } from "luxon";
import type { EntityPeriod } from "@/lib/entities";

// Like Luxon's Interval type, but not half-open.
export interface Interval {
  start: DateTime;
  end: DateTime;
}

export type Period = EntityPeriod;

export const PERIODS: Period[] = [
  "hour",
  "day",
  "week",
  "month",
  "year",
] as const;

export function toHumanDuration(
  ticks: number,
  showSymbolForZero = true,
  symbolForZero = "-",
): string {
  return ticks === 0 && showSymbolForZero
    ? symbolForZero
    : humanizeDuration(ticks / 10_000, {
        unitCount: 2,
        hideYear: true,
      });
}

export function toHumanDurationFull(
  ticks: number,
  showSymbolForZero = true,
  symbolForZero = "-",
): string {
  return ticks === 0 && showSymbolForZero
    ? symbolForZero
    : humanizeDuration(ticks / 10_000, {
        hideYear: true,
      });
}

export function dateTimeToTicks(dt: DateTime): number {
  return unixMillisToTicks(dt.toMillis());
}

export function unixMillisToTicks(ts: number): number {
  return (ts + 62_135_596_800_000) * 10_000;
}

export function ticksToUnixMillis(ticks: number): number {
  return ticks / 10_000 - 62_135_596_800_000;
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

export function periodToDuration(period: Period): Duration {
  return Duration.fromObject({ [period]: 1 });
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

export function toHumanDateTimeFull(dt: DateTime): string {
  return dt.toFormat("LLL dd, y hh:mm:ss a");
}

export const hour24Formatter = (dateTime: DateTime) =>
  dateTime.toFormat("HHmm");
export const weekDayFormatter = (dateTime: DateTime) =>
  dateTime.toFormat("EEE");
export const monthDayFormatter = (dateTime: DateTime) =>
  dateTime.toFormat("dd");

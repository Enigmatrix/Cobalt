import type { EntityPeriod } from "@/lib/entities";
import { DateTime, Duration } from "luxon";
import humanizeDuration from "pretty-ms";

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

export const TICKS_PER_MILLISECOND = 10_000;
export const TICKS_EPOCH_DIFF = 62_135_596_800 * 1_000; // 01-01-0001 00:00:00 to 1970-01-01 00:00:00 in milliseconds

export function toHumanDuration(
  ticks: number,
  showSymbolForZero = true,
  symbolForZero = "-",
): string {
  return ticks === 0 && showSymbolForZero
    ? symbolForZero
    : humanizeDuration(ticks / TICKS_PER_MILLISECOND, {
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
    : humanizeDuration(ticks / TICKS_PER_MILLISECOND, {
        hideYear: true,
      });
}

export function dateTimeToTicks(dt: DateTime): number {
  return unixMillisToTicks(dt.toMillis());
}

export function unixMillisToTicks(ts: number): number {
  return (ts + TICKS_EPOCH_DIFF) * TICKS_PER_MILLISECOND;
}

export function ticksToUnixMillis(ticks: number): number {
  return ticks / TICKS_PER_MILLISECOND - TICKS_EPOCH_DIFF;
}

export function durationToTicks(ts: Duration): number {
  return ts.toMillis() * TICKS_PER_MILLISECOND;
}

export function ticksToDuration(ts: number): Duration {
  return Duration.fromMillis(ts / TICKS_PER_MILLISECOND);
}

export function ticksToDateTime(ticks: number): DateTime {
  return DateTime.fromMillis(ticks / TICKS_PER_MILLISECOND - TICKS_EPOCH_DIFF);
}

export function periodToDuration(period: Period): Duration {
  return Duration.fromObject({ [period]: 1 });
}

export function toHumanDateTime(dt: DateTime): string {
  // if time part is 00:00:00, then return date part only
  if (dt.toFormat("HH:mm:ss") === "00:00:00") {
    return dt.toFormat("dd LLL y");
  }
  if (dt.toFormat("ss") === "00") {
    return dt.toFormat("dd LLL y hh:mm a");
  }
  return dt.toFormat("dd LLL y hh:mm:ss a");
}

export function toHumanDateTimeFull(dt: DateTime): string {
  return dt.toFormat("dd LLL y hh:mm:ss a");
}

export function toHumanInterval(today: DateTime, interval: Interval): string {
  const known = toKnownHumanInterval(today, interval);
  if (known) return known;
  return (
    toHumanDateTime(interval.start) + " - " + toHumanDateTime(interval.end)
  );
}

export function toKnownHumanInterval(
  today: DateTime,
  interval: Interval,
): string | undefined {
  const period = PERIODS.reverse().find((p) =>
    isIntervalMatchingPeriod(interval, p),
  );
  if (!period) return undefined;
  return periodTitles[period](today, interval);
}

export function isIntervalMatchingPeriod(
  interval: Interval,
  period: Period,
): boolean {
  const isStart = interval.start.startOf(period).equals(interval.start);
  if (!isStart) return false;
  // is end matching period?
  const isEnd = interval.start.plus({ [period]: 1 }).equals(interval.end);
  return isEnd;
}

function periodTitleHour(today: DateTime, interval: Interval): string {
  const dt = interval.start;
  const format =
    dt.toFormat("y LLL dd") === today.toFormat("y LLL dd")
      ? "HH:mm"
      : "dd LLL y hh:mm a";
  return dt.toFormat(format);
}

function periodTitleDay(today: DateTime, interval: Interval): string {
  const dt = interval.start;
  if (today.startOf("day").equals(dt)) return "Today";
  if (today.startOf("day").minus({ day: 1 }).equals(dt)) return "Yesterday";
  const format = dt.year === today.year ? "dd LLL" : "dd LLL y";
  return dt.toFormat(format);
}

function periodTitleWeek(today: DateTime, interval: Interval): string {
  const dt = interval.start;
  if (today.startOf("week").equals(dt)) return "This Week";
  if (today.startOf("week").minus({ week: 1 }).equals(dt)) return "Last Week";
  const format =
    dt.year === today.year && dt.endOf("week").year === today.year
      ? "dd LLL"
      : "dd LLL y";
  return dt.toFormat(format) + " - " + dt.endOf("week").toFormat(format);
}

function periodTitleMonth(today: DateTime, interval: Interval): string {
  const dt = interval.start;
  if (today.startOf("month").equals(dt)) return "This Month";
  if (today.startOf("month").minus({ month: 1 }).equals(dt))
    return "Last Month";
  const format = dt.year === today.year ? "LLL" : "LLL y";
  return dt.toFormat(format);
}

function periodTitleYear(today: DateTime, interval: Interval): string {
  const dt = interval.start;
  if (today.startOf("year").equals(dt)) return "This Year";
  if (today.startOf("year").minus({ year: 1 }).equals(dt)) return "Last Year";
  return dt.toFormat("y");
}

const periodTitles = {
  hour: periodTitleHour,
  day: periodTitleDay,
  week: periodTitleWeek,
  month: periodTitleMonth,
  year: periodTitleYear,
} as const;

export const hour24Formatter = (dateTime: DateTime) =>
  dateTime.toFormat("HHmm");
export const weekDayFormatter = (dateTime: DateTime) =>
  dateTime.toFormat("EEE");
export const monthDayFormatter = (dateTime: DateTime) =>
  dateTime.toFormat("dd");

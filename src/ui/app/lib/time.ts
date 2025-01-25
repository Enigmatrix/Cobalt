import humanizeDuration from "pretty-ms";

export function toHumanDuration(dur: number): string {
  return humanizeDuration(dur / 10_000, { unitCount: 2 });
}

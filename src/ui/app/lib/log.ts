import { invoke } from "@tauri-apps/api/core";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type LogArg = any;

export function log(
  level: string,
  message: string,
  file: string,
  line: number,
  col: number,
  args: LogArg,
) {
  // this is a promise, but we pretend it's synchronous
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  invoke("log", { level, message, file, line, col, args });
}

function getCallerLocation() {
  const e = new Error();
  const stack = e.stack?.split("\n")[3];
  const match = stack?.match(/\(https?:\/\/[^/]*\/(.*):(\d+):(\d+)\)$/);

  return {
    file: (match?.[1] ?? "unknown").split("?")[0],
    line: parseInt(match?.[2] ?? "0"),
    col: parseInt(match?.[3] ?? "0"),
  };
}

export function error(message: string, args?: LogArg) {
  const { file, line, col } = getCallerLocation();
  log("error", message, file, line, col, args);
}

export function warn(message: string, args?: LogArg) {
  const { file, line, col } = getCallerLocation();
  log("warn", message, file, line, col, args);
}

export function info(message: string, args?: LogArg) {
  const { file, line, col } = getCallerLocation();
  log("info", message, file, line, col, args);
}

export function debug(message: string, args?: LogArg) {
  const { file, line, col } = getCallerLocation();
  log("debug", message, file, line, col, args);
}

export function trace(message: string, args?: LogArg) {
  const { file, line, col } = getCallerLocation();
  log("trace", message, file, line, col, args);
}

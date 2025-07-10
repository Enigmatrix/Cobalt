import { useEffect, useState } from "react";

/**
 * Returns the last non-null value of a value that can be null. When the hook is created
 * either value or defaultValue MUST be non-null.
 * @param value - The value to return.
 * @param defaultValue - The default value to return if the value is null.
 * @returns The last non-null value of the value.
 */
export function useLastNonNull<T>(value: T | null, defaultValue?: T): T {
  const [inner, setInner] = useState<T>(() => {
    if (value === null && defaultValue === undefined) {
      throw new Error(
        "useLastNonNull: value and defaultValue are both null/undefined",
      );
    }
    return value ?? defaultValue!;
  });
  useEffect(() => {
    if (value !== null) {
      setInner(value);
    }
  }, [value]);
  return inner;
}

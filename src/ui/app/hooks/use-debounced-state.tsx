import { error } from "@/lib/log";
import { useEffect, useState } from "react";
import { useDebouncedCallback } from "use-debounce";

export function useDebouncedState<T>(
  initialState: T,
  updateFn: (state: T) => Promise<void>,
  debounceMs: number,
) {
  const [state, setStateInner] = useState(initialState);
  const debouncedUpdate = useDebouncedCallback(updateFn, debounceMs);
  useEffect(() => {
    debouncedUpdate(state)?.catch((err) => {
      console.error(err);
      error("Failed to update debounced state", err);
    });
  }, [state, debouncedUpdate]);

  return [state, setStateInner] as const;
}

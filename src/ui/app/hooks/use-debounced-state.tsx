import { useCallback, useState } from "react";
import { useDebouncedCallback } from "use-debounce";

export function useDebouncedState<T>(
  initialState: T,
  updateFn: (state: T) => Promise<void>,
  debounceMs: number,
) {
  const [state, setStateInner] = useState(initialState);
  // TODO: this doesn't really debounce the promise itself, just the promise returner function
  const debouncedUpdate = useDebouncedCallback(updateFn, debounceMs);

  const setState = useCallback(
    async (state: T) => {
      setStateInner(state);
      await debouncedUpdate(state);
    },
    [setStateInner, debouncedUpdate],
  );

  return [state, setState, setStateInner] as const;
}

import {
  useCallback,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";

function getHistoryValue<S>(key: string): S | undefined {
  return (history.state as Record<string, S | undefined> | undefined)?.[key];
}

function updateHistoryValue<S>(key: string, value: S) {
  history.replaceState(
    {
      ...history.state,
      [key]: value,
    },
    "",
  );
}

/**
 * Like {@link useState}, but persists in the history state. This is useful for
 * back-button navigation. `key` is the key to use in the history state
 * and must be unique per page. `key` should not change after the first render.
 */
export function useHistoryState<S>(
  initialState: S | (() => S),
  key: string | undefined,
): [S, Dispatch<SetStateAction<S>>] {
  const [rawState, rawSetState] = useState<S>(() => {
    if (key) {
      // If the state is already set in history, return it
      const historyValue = getHistoryValue<S>(key);
      if (historyValue) return historyValue;
    }

    // Otherwise, set the state and return the initial value,
    // making sure to update the history state.
    const retValue =
      typeof initialState === "function"
        ? (initialState as () => S)()
        : initialState;

    if (key) {
      updateHistoryValue(key, retValue);
    }

    return retValue;
  });

  const setState = useCallback(
    (value: SetStateAction<S>) => {
      rawSetState((prevValue) => {
        const newValue =
          typeof value === "function"
            ? (value as (prev: S) => S)(prevValue)
            : value;
        if (key) {
          updateHistoryValue(key, newValue);
        }
        return newValue;
      });
    },
    [rawSetState, key],
  );

  return [rawState, setState];
}

/**
 * Same as {@link useHistoryState}, but never changes the returned value when
 * the setter is called - instead only the history state is updated. So it's
 * more similar to `useRef` than `useState`.
 */
export function useHistoryRef<S>(
  initialState: S | (() => S),
  key: string | undefined,
): [S, Dispatch<SetStateAction<S>>] {
  const [rawState] = useState<S>(() => {
    if (key) {
      // If the state is already set in history, return it
      const historyValue = getHistoryValue<S>(key);
      if (historyValue) return historyValue;
    }

    // Otherwise, set the state and return the initial value,
    // making sure to update the history state.
    const retValue =
      typeof initialState === "function"
        ? (initialState as () => S)()
        : initialState;

    if (key) {
      updateHistoryValue(key, retValue);
    }
    return retValue;
  });

  const setState = useCallback(
    (value: SetStateAction<S>) => {
      if (!key) return;
      const historyValue = getHistoryValue<S>(key);

      const newValue =
        typeof value === "function"
          ? (value as (prev: S) => S)(historyValue!)
          : value;

      updateHistoryValue(key, newValue);
    },
    [key],
  );

  return [rawState, setState];
}
